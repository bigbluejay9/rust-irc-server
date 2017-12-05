mod codec;
mod messages;
mod data;
mod processor;
mod stats;

use chrono;
use hostname;

use std::io;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;

// A union of socket events and server-wide events.
#[derive(Debug)]
enum ClientEvent {
    Socket(String),
    Broadcast(String),
}

pub fn start(local_addr: SocketAddr, http: Option<SocketAddr>) {
    let mut event_loop = Core::new().unwrap();
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();
    let lis = TcpListener::bind(&local_addr, &handle).unwrap();

    let server = Arc::new(Mutex::new(data::Server::new(
        chrono::offset::Utc::now(),
        "0.1".to_string(),
    )));

    stats::start_stats_server(http, &handle, Arc::clone(&server));

    let srv = lis.incoming().for_each(|(stream, addr)| {
        let server = Arc::clone(&server);
        // Create client connection.
        handle.spawn(thread_pool.spawn_fn(move || {
            let socket = data::SocketPair {
                local: local_addr,
                remote: addr,
            };
            let client = Arc::new(Mutex::new(
                data::Client::new(socket.clone(), Arc::clone(&server)),
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
                    let server_prefix =
                        Some(hostname::get_hostname().expect("unable to get hostname"));

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
                            processor::process_message(
                                Arc::clone(&client_handle),
                                server_prefix,
                                message,
                            )
                        }

                        ClientEvent::Broadcast(_b) => {
                            unimplemented!("Unimplemented branch arm - broadcast message.");
                        }
                    };
                    future::ok(res)
                })
                .then(|messages: Result<Vec<messages::Message>, _>| {
                    // ** Serialization future.
                    if messages.is_err() {
                        return future::err(messages.err().unwrap());
                    }
                    let mut result = Vec::new();
                    // TODO(lazau): Perform 512 max line size here.
                    for m in messages.unwrap() {
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
