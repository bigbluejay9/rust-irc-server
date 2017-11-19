mod proto;
mod messages;

use num_cpus;
use hostname;

use std::collections::HashSet;
use std::io;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::reactor::Core;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_io::AsyncRead;

use tokio_proto::TcpServer;
use tokio_service::Service;

pub fn start(addr: SocketAddr) -> io::Result<()> {
    let mut event_loop = Core::new()?;
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();
    let lis = TcpListener::bind(&addr, &handle)?;

    let server = lis.incoming().for_each(|(stream, addr)| {
        let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel(1);
        tx.send("a message from the queue.".to_string())
            .wait()
            .unwrap();
        debug!("Accepting connection: {:?}, {:?}.", stream, addr);
        handle.spawn(
            thread_pool
                .spawn(
                    stream
                        .framed(proto::Utf8CrlfCodec)
                        .select(rx.map_err(
                            |_| io::Error::new(io::ErrorKind::Other, "rx error"),
                        ))
                        .for_each(|s| {
                            debug!("got {:?} frame.", s);
                            future::ok(())
                        }),
                )
                .then(|_| future::ok(())),
        );
        future::ok(())
    });

    event_loop.run(server)

    /*let cpus = num_cpus::get();
    let server_data = Arc::new(Mutex::new(ServerData {
        hostname: hostname::get_hostname().expect("cannot get server hostname"),
        nicknames: HashSet::new(),
        open_connections: 0,
    }));
    debug!("Starting server on: {:?}, with {:?} cpus.", addr, cpus);

    let mut server = TcpServer::new(proto::IRCProto, addr);
    server.threads(cpus);
    server.serve(move || {
        debug!("New econnection.");
        Ok(IRC {
            server: Arc::clone(&server_data),
            connection: Arc::new(ConnectionData {
                nick: None,
                remote: None,
            }),
        })
    });*/
}

#[derive(Debug)]
struct ServerData {
    hostname: String,
    nicknames: HashSet<String>,
    open_connections: u64,
}

#[derive(Debug)]
struct ConnectionData<'conn> {
    nick: Option<&'conn String>,
    remote: Option<SocketAddr>,
}

struct IRC<'conn> {
    server: Arc<Mutex<ServerData>>,
    connection: Arc<ConnectionData<'conn>>,
}

impl<'conn> Service for IRC<'conn> {
    type Request = String;
    type Response = Option<String>;
    type Error = io::Error;
    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        debug!(
            "Processing [{:?}].\nServer state: {:?}.\nConnection state: {:?}.\n",
            req,
            self.server,
            self.connection
        );

        let message;
        match req.parse::<messages::Message>() {
            Ok(m) => message = m,
            Err(ref e) => {
                warn!("Bad client message [{:?}]: {:?}.", req, e);
                return future::ok(None);
            }
        };

        debug!("Command: {:?}.", message);
        future::ok(match message.command {
            messages::Command::NICK => Some(req),
            messages::Command::USER => Some(req),
            u @ _ => {
                error!("Response to {:?} not yet implemented.", u);
                None
            }
        })
    }
}

impl<'conn> Drop for IRC<'conn> {
    fn drop(&mut self) {
        debug!("Connection closing.\nServer data {:?}.", self.server);
    }
}
