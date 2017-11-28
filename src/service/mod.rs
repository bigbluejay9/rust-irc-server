mod codec;
mod messages;

use hostname;
use chrono;

use std::collections::HashSet;
use std::io;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use futures::*;
use futures::sync::mpsc;
use futures::stream::Stream;
use futures_cpupool::CpuPool;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_io::AsyncRead;

pub fn start(addr: SocketAddr) {
    let mut event_loop = Core::new().unwrap();
    let handle = event_loop.handle();
    let thread_pool = CpuPool::new_num_cpus();
    let lis = TcpListener::bind(&addr, &handle).unwrap();

    let server = Arc::new(Mutex::new(Server {
        created: chrono::offset::Utc::now(),
        version: "0.1".to_string(),
        hostname: hostname::get_hostname().expect("unable to get hostname"),
        nicknames: HashSet::new(),
        clients: Vec::new(),
    }));

    let srv = lis.incoming().for_each(|(stream, addr)| {
        let server = Arc::clone(&server);
        // Create client connection.
        handle.spawn(thread_pool.spawn_fn(move || {
            let client = Arc::new(Mutex::new(ConnectionData {
                remote_addr: addr,
                nick: None,
                user: None,
            }));

            // TODO(lazau): How large should this buffer be?
            // Note: should penalize slow clients.
            let (tx, rx) = mpsc::channel(5);
            server.lock().unwrap().clients.push(tx);

            // Refcount for handle future.
            let client_handle = Arc::clone(&client);
            let server_handle = Arc::clone(&server);
            // Refcount for cleanup future.
            let client_cleanup = Arc::clone(&client);
            let server_cleanup = Arc::clone(&server);

            let (sink, stream) = stream.framed(codec::Utf8CrlfCodec).split();
            stream.map(|s| ClientEvent::Socket(s) )
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
                                //Err(e) => return future::err(e),
                            };
                            process_message(
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
                .then( // ** Serialization future.
                    |messages: Result<Vec<messages::Message>, _>| -> future::FutureResult<Vec<String>, _> {
                        let mut result = Vec::new();
                        // TODO(lazau): Perform 512 max line size here.
                        for m in messages {
                            // TODO(lazau): Convert serialization error to future::err.
                            result.push(messages::to_string(&m).unwrap());
                        }
                        future::ok(result)
                    },
                )
                .forward(sink)
                .then(move |e: Result<(_, _), io::Error>| {
                    // ** Cleanup future.
                    if let Err(e) = e {
                        warn!("Connection error: {:?}.", e);
                    }

                    // TODO(lazau): Perform connection cleanup here.
                    let client = client_cleanup.lock().unwrap();
                    if let Some(ref nick) = client.nick {
                        if !server_cleanup.lock().unwrap().nicknames.remove(nick) {
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

    match event_loop.run(srv) {
        Err(e) => error!("Server failure: {:?}.", e),
        _ => {}
    };
}

#[derive(Debug)]
struct Server {
    created: chrono::DateTime<chrono::Utc>,
    version: String,
    hostname: String,
    nicknames: HashSet<String>,
    // Receiver may be gone, must check for send errors!
    clients: Vec<mpsc::Sender<String>>,
}

// A union of socket events and server-wide events.
#[derive(Debug)]
enum ClientEvent {
    Socket(String),
    Broadcast(String),
}

#[derive(Debug)]
struct UserData {
    username: String,
    realname: String,
}

#[derive(Debug)]
struct ConnectionData {
    remote_addr: SocketAddr,
    nick: Option<String>,
    user: Option<UserData>,
}

fn process_message(
    server: Arc<Mutex<Server>>,
    client: Arc<Mutex<ConnectionData>>,
    server_prefix: Option<String>,
    req: messages::Message,
) -> Vec<messages::Message> {
    trace!(
        "Processing request [{:?}].\nClient state: {:?}.\nServer state: {:?}.\nServer prefix: {:?}.",
        req,
        client,
        server,
        server_prefix
    );

    match req.command {
        messages::Command::Req(r) => {
            match r {
                messages::Request::NICK { nickname: nick } => {
                    // TODO(lazau): Validate nick based on
                    // https://tools.ietf.org/html/rfc2812#section-2.3.1.
                    let mut server = server.lock().unwrap();
                    if !server.nicknames.insert(nick.clone()) {
                        return vec![
                            messages::Message {
                                prefix: server_prefix,
                                command: messages::Command::Resp(
                                    messages::Response::ERR_NICKNAMEINUSE
                                ),
                            },
                        ];
                    }
                    let mut client = client.lock().unwrap();
                    client.nick = Some(nick.clone());

                    maybe_welcome_sequence(&server, &client, server_prefix)
                }

                messages::Request::USER {
                    username,
                    mode,
                    unused,
                    realname,
                } => {
                    /*if message.params[1] != "0" || message.params[2] != "*" {
                        warn!(
                            "USER command {:?}, second and third param unknown.",
                            message.command
                        );
                    }*/

                    let mut client = client.lock().unwrap();
                    client.user = Some(UserData {
                        username: username,
                        realname: realname,
                    });

                    maybe_welcome_sequence(&*server.lock().unwrap(), &client, server_prefix)
                }

                u @ _ => {
                    error!("Response to {:?} not yet implemented.", u);
                    Vec::new()
                }
            }
        }
        r @ _ => {
            error!("{:?} isn't a client request. Dropping", r);
            Vec::new()
        }
    }
}

// Returns WELCOME sequence if client has successfully registered.
fn maybe_welcome_sequence(
    server: &Server,
    client: &ConnectionData,
    server_prefix: Option<String>,
) -> Vec<messages::Message> {
    if client.user.is_none() || client.nick.is_none() {
        return Vec::new();
    }

    vec![
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_WELCOME {
                message: Some(
                    "Welcome to the <networkname> Network, <nick>[!<user>@<host>]".to_string(),
                ),
            }),
        },
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_YOURHOST),
        },
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_CREATED),
        },
        messages::Message {
            prefix: server_prefix.clone(),
            command: messages::Command::Resp(messages::Response::RPL_MYINFO),
        },
    ]
}
