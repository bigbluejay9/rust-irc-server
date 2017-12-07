mod codec;
mod messages;
mod server;
mod stats;
mod templates;
mod client;

use chrono;

use std::{io, fmt};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;

// Used to identify clients.
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
enum ClientEvent {
    Socket(String),
    Broadcast(Arc<Broadcast>),
}

#[derive(Debug)]
pub enum Broadcast {
    Join(client::UserPrefix, String),
    Part,
    PrivateMessage,
}

pub fn start(local_addr: SocketAddr, http: Option<SocketAddr>) {
    let mut event_loop = Core::new().unwrap();
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();
    let lis = TcpListener::bind(&local_addr, &handle).unwrap();

    // Immutable configuration.
    let configuration = Arc::new(server::Configuration::new(
        chrono::offset::Utc::now(),
        "0.1".to_string(),
    ));
    let server = Arc::new(Mutex::new(server::Server::new()));

    stats::start_stats_server(
        http,
        &handle,
        Arc::clone(&configuration),
        Arc::clone(&server),
    );

    let srv = lis.incoming().for_each(|(stream, addr)| {
        let configuration = Arc::clone(&configuration);
        let server = Arc::clone(&server);
        // Create client connection.
        handle.spawn(thread_pool.spawn_fn(move || {
            let socket = SocketPair {
                local: local_addr,
                remote: addr,
            };
            let client = Arc::new(Mutex::new(
                client::Client::new(socket.clone(), Arc::clone(&server)),
            ));

            // TODO(lazau): How large should this buffer be?
            // Note: should penalize slow clients.
            let (tx, rx) = mpsc::channel(5);
            server.lock().unwrap().insert_client(
                &socket,
                Arc::clone(&client),
                tx,
            );

            // Refcount for handle future.
            let client_handle = Arc::clone(&client);
            // Refcount for cleanup future.
            let client_cleanup = Arc::clone(&client);
            // Refcount for serialization future.
            let configuration_serialization = Arc::clone(&configuration);

            let (sink, stream) = stream.framed(codec::Utf8CrlfCodec).split();
            stream
                .map(|s| ClientEvent::Socket(s))
                .select(rx.then(|e| {
                    Ok(ClientEvent::Broadcast(
                        e.expect("client channel rx error. should never happen."),
                    ))
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
                        ClientEvent::Socket(s) => {
                            let message = match s.parse::<messages::Message>() {
                                Ok(m) => m,
                                // TODO(lazau): Maybe do some additional error processing here?
                                Err(e) => {
                                    warn!("Failed to parse {}: {:?}.", s, e);
                                    return future::ok(Vec::new());
                                }
                            };
                            debug!("Request [{:?}].", message);
                            client::process_message(
                                Arc::clone(&configuration),
                                Arc::clone(&client_handle),
                                message,
                            )
                        }

                        ClientEvent::Broadcast(b) => {
                            debug!("Broadcast [{:?}].", b);
                            client::process_broadcast(
                                Arc::clone(&configuration),
                                Arc::clone(&client_handle),
                                b,
                            )
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
                            m.prefix = Some(configuration_serialization.hostname.clone());
                        }
                        // TODO(lazau): Convert serialization error to future::err.
                        result.push(format!("{}", m));
                    }
                    future::ok(result)
                })
                .forward(sink)
                .then(move |e: Result<(_, _), io::Error>| {
                    // ** Cleanup future.
                    let client = client_cleanup.lock().unwrap();
                    client.server.lock().unwrap().remove_client(&client);
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
