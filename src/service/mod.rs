mod codec;
mod messages;
mod server;
mod stats;
mod templates;
mod connection;
mod channel;
mod user;

use chrono;

use std::{io, fmt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

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

pub fn start(local_addr: SocketAddr, http: Option<SocketAddr>) {
    let mut event_loop = Core::new().unwrap();
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();
    let lis = TcpListener::bind(&local_addr, &handle).unwrap();

    let server = Arc::new(server::Server::new(
        chrono::offset::Utc::now(),
        "0.1".to_string(),
    ));
    let connections = Arc::new(Mutex::new(HashMap::new()));

    stats::start_stats_server(http, &handle, Arc::clone(&server), Arc::clone(&connections));

    let srv = lis.incoming().for_each(|(stream, addr)| {
        let server = Arc::clone(&server);
        let connections = Arc::clone(&connections);
        // Create connection connection.
        handle.spawn(thread_pool.spawn_fn(move || {
            let socket = SocketPair {
                local: local_addr,
                remote: addr,
            };

            // TODO(lazau): How large should this buffer be?
            // Note: should penalize slow connections.
            let (tx, rx) = mpsc::channel(20);
            let connection = Arc::new(Mutex::new(connection::Connection::new(
                socket.clone(),
                Arc::clone(&server),
                tx,
            )));
            connections.lock().unwrap().insert(
                socket.clone(),
                Arc::clone(&connection),
            );

            // Refcount for handle future.
            let connection_handle = Arc::clone(&connection);
            // Refcount for cleanup future.
            let connection_cleanup = Arc::clone(&connection);
            let connections_cleanup = Arc::clone(&connections);
            // Refcount for serialization future.
            let server_serialization = Arc::clone(&server);

            let (sink, stream) = stream.framed(codec::Utf8CrlfCodec).split();
            stream
                .map(|s| ConnectionEvent::Socket(s))
                .select(rx.then(|e| {
                    Ok(ConnectionEvent::Broadcast(e.expect(
                        "connection channel rx error. should never happen.",
                    )))
                }))
                .then(move |event| {
                    // ** Process future.
                    trace!("Connection event: {:?}.", event);
                    if event.is_err() {
                        let err = event.err().unwrap();
                        error!("Unexpected upstream error: {:?}.", err);
                        return future::err(err);
                    }

                    let res = match event.unwrap() {
                        ConnectionEvent::Socket(s) => {
                            let message = match s.parse::<messages::Message>() {
                                Ok(m) => m,
                                // TODO(lazau): Maybe do some additional error processing here?
                                Err(e) => {
                                    warn!("Failed to parse {}: {:?}.", s, e);
                                    return future::ok(Vec::new());
                                }
                            };
                            debug!("Request [{:?}].", message);
                            connection_handle.lock().unwrap().process_message(message)
                        }

                        ConnectionEvent::Broadcast(b) => {
                            debug!("Broadcast [{:?}].", b);
                            connection_handle.lock().unwrap().process_broadcast(b)
                        }
                    };
                    debug!("Response [{:?}].", res);
                    future::ok(res)
                })
                .then(move |messages: Result<Vec<messages::Message>, _>| {
                    // ** Serialization future.
                    if messages.is_err() {
                        return future::err(messages.err().unwrap());
                    }
                    let mut result = Vec::new();
                    // TODO(lazau): Perform 512 max line size here.
                    for mut m in messages.unwrap() {
                        if m.prefix.is_none() {
                            m.prefix = Some(server_serialization.hostname.clone());
                        }
                        // TODO(lazau): Convert serialization error to future::err.
                        result.push(format!("{}", m));
                    }
                    future::ok(result)
                })
                .forward(sink)
                .then(move |e: Result<(_, _), io::Error>| {
                    // ** Cleanup future.
                    let mut connection = connection_cleanup.lock().unwrap();
                    connection.disconnect();
                    connections.lock().unwrap().remove(&connection.socket);
                    if let Err(e) = e {
                        warn!("Connection error: {:?}.", e);
                    }
                    Ok(())
                })
        }));
        Ok(())
    });

    debug!("Starting IRC server at {:?}.", local_addr);
    match event_loop.run(srv) {
        Err(e) => error!("Server failure: {:?}.", e),
        _ => {}
    };
}
