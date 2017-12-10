mod codec;
mod messages;
mod server;
mod shared_state;
mod stats;
mod connection;
mod channel;
mod user;

use chrono;
use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;
use hostname;
use std::{io, fmt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use super::configuration;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;

// Used to identify connections.
// Server is represented by (local, local) pair.
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct SocketPair {
    pub local: SocketAddr,
    pub remote: SocketAddr,
}

impl fmt::Display for SocketPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({} : {})", self.local, self.remote)
    }
}

// A union of socket events and server-wide events.
#[derive(Debug)]
enum ConnectionEvent {
    Socket(String),
    Broadcast(Arc<Broadcast>),
}

#[derive(Debug)]
pub enum Broadcast {
    // (User, Channel).
    Join(user::Identifier, String),
    // (User, Channel, Message)
    Part(user::Identifier, String, Option<String>),
    PrivateMessage,
}

pub fn start(configuration: Arc<configuration::Configuration>) {
    let mut event_loop = Core::new().unwrap();
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();

    let shared_state = Arc::new(shared_state::SharedState::new(
        chrono::offset::Utc::now(),
        hostname::get_hostname().unwrap(),
        configuration,
    ));
    let server = server::new(Arc::clone(&shared_state), thread_pool.clone());

    let insecure_addr = shared_state.configuration.insecure_listen_address.unwrap().clone();
    let insecure_lis = TcpListener::bind(
    insecure_addr
        &handle,
    ).unwrap();

    stats::start_stats_server(
        shared_state
            .configuration
            .http_debug_listen_address
            .as_ref()
            .unwrap(),
        &handle,
        Arc::clone(&configuration),
        Arc::clone(&shared_state),
        Arc::clone(&server),
        Arc::clone(&connections),
    );

    let srv =
        insecure_lis.incoming().for_each(|(stream, addr)| {
            // Create connection connection.
            thread_pool
                .spawn_fn(connection::handle_new_connection(stream, (insecure_addr.clone(), addr), Arc::clone(&server), Arc::clone(&connections)))
                .forget();
            Ok(())
        });

    debug!("Starting IRC server at {:?}.", insecure_addr);
    match event_loop.run(srv) {
        Err(e) => error!("Server failure: {:?}.", e),
        _ => {}
    };
}
