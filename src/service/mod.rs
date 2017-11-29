mod codec;
mod messages;
mod data;
mod processor;

use hostname;
use chrono;

use std::collections::{HashSet, HashMap};
use std::io;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpListener;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::io::write_all;

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

    let server = Arc::new(Mutex::new(data::Server {
        created: chrono::offset::Utc::now(),
        version: "0.1".to_string(),
        hostname: hostname::get_hostname().expect("unable to get hostname"),
        nicknames: HashSet::new(),
        connections: HashMap::new(),
    }));

    start_stats_server(http, &handle);

    let srv = lis.incoming().for_each(|(stream, addr)| {
        let server = Arc::clone(&server);
        // Create client connection.
        handle.spawn(thread_pool.spawn_fn(move || {
            // TODO(lazau): How large should this buffer be?
            // Note: should penalize slow clients.
            let client = Arc::new(Mutex::new(data::Connection {
                local_addr: local_addr,
                remote_addr: addr,
                nick: None,
                user: None,
            }));

            // TODO(lazau): How large should this buffer be?
            // Note: should penalize slow clients.
            let (tx, rx) = mpsc::channel(5);
            server.lock().unwrap().connections.insert(
                (local_addr, addr),
                tx,
            );

            // Refcount for handle future.
            let client_handle = Arc::clone(&client);
            let server_handle = Arc::clone(&server);
            // Refcount for cleanup future.
            let client_cleanup = Arc::clone(&client);
            let server_cleanup = Arc::clone(&server);

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
                                Arc::clone(&server_handle),
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
                        result.push(messages::to_string(&m).unwrap());
                    }
                    future::ok(result)
                })
                .forward(sink)
                .then(move |e: Result<(_, _), io::Error>| {
                    // ** Cleanup future.
                    let client = client_cleanup.lock().unwrap();
                    let mut server = server_cleanup.lock().unwrap();
                    server.connections.remove(
                        &(client.local_addr, client.remote_addr),
                    );

                    if let Err(e) = e {
                        warn!("Connection error: {:?}.", e);
                    }

                    if let Some(ref nick) = client.nick {
                        if !server.nicknames.remove(nick) {
                            error!(
                                "Trying to self remove nick {:?}, but it didn't exist.",
                                nick
                            );
                        }
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

fn start_stats_server(http: Option<SocketAddr>, handle: &Handle) {

    if let Some(addr) = http {
        debug!("Staring debug HTTP server at {:?}.", addr);
        let lis = TcpListener::bind(&addr, &handle).unwrap();
        let srv = lis.incoming().for_each(|(mut stream, _addr)| {
            write_all(stream, "HTTP/1.0 200 OK\r\n\r\nOKOKOK".as_bytes())
        });
        //.map_err(|e| ());

        handle.spawn(srv);
    }
}
