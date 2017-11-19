mod proto;
mod messages;

use num_cpus;

use std::io;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use futures::{future, Future};

use tokio_proto::TcpServer;
use tokio_service::Service;

pub fn start(addr: SocketAddr) {
    let cpus = num_cpus::get();
    let server_data = Arc::new(Mutex::new(ServerData { open_connections: 0 }));
    debug!("Starting server on: {:?}, with {:?} cpus.", addr, cpus);
    let mut server = TcpServer::new(proto::IRCProto, addr);
    server.threads(cpus);
    server.serve(move || {
        server_data.lock().unwrap().open_connections += 1;
        debug!("New service created!");
        Ok(IRC {
            server: Arc::clone(&server_data),
            connection: Arc::new(ConnectionData { remote: None }),
        })
    });
}

#[derive(Debug)]
struct ServerData {
    open_connections: u64,
}

#[derive(Debug)]
struct ConnectionData {
    remote: Option<SocketAddr>,
}

struct IRC {
    server: Arc<Mutex<ServerData>>,
    connection: Arc<ConnectionData>,
}

impl Service for IRC {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        debug!(
            "Processing request.\nServer state: {:?}.\n Conn state: {:?}.\n",
            self.server,
            self.connection
        );
        Box::new(future::ok(req))
    }
}

impl Drop for IRC {
    fn drop(&mut self) {
        self.server.lock().unwrap().open_connections -= 1;
        debug!("Connection closing.\nServer data {:?}.", self.server);
    }
}
