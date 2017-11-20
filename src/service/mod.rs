mod codec;
mod messages;

use hostname;

use std::str;
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
        hostname: hostname::get_hostname().expect("unable to get hostname"),
        nicknames: HashSet::new(),
        clients: Vec::new(),
    }));

    let srv = lis.incoming().for_each(|(stream, addr)| {
        let server = Arc::clone(&server);
        // Create client connection.
        handle.spawn(thread_pool.spawn_fn(move || {
            debug!("Accepting connection: {:?}, {:?}.", stream, addr);

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
            stream
                .map(|s| ClientEvent::Socket(s))
                .select(rx.map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "can never happen")
                }))
                .then(move |event| {
                    debug!("Connection event: {:?}.", event);
                    if event.is_err() {
                        let err = event.err().unwrap();
                        warn!("Upstream error: {:?}.", err);
                        return future::err(err);
                    }

                    // TODO(lazau): Wrap result in a Result type, log issues.
                    let result = match event.unwrap() {
                        ClientEvent::Socket(s) => {
                            process_message(
                                Arc::clone(&server_handle),
                                Arc::clone(&client_handle),
                                s,
                            )
                        }
                        // TODO(lazau): implement.
                        ClientEvent::Broadcast(b) => Some(
                            format!("got broadcast message {:?}.", b),
                        ),
                    };
                    future::ok(result)
                })
                .forward(sink)
                .then(move |e| {
                    if let Err(e) = e {
                        warn!("Connection error: {:?}.", e);
                    }

                    // TODO(lazau): Perform connection cleanup here.
                    let client = client_cleanup.lock().unwrap();
                    warn!("Trying to remove {:?}.", client);
                    if let Some(ref nick) = client.nick {
                        if !server_cleanup.lock().unwrap().nicknames.remove(nick) {
                            warn!(
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
    hostname: String,
    nicknames: HashSet<String>,
    // Receiver may be gone, must check for send errors!
    clients: Vec<mpsc::Sender<ClientEvent>>,
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
    req: String,
) -> Option<String> {
    debug!(
        "Processing request [{:?}].\nClient state: {:?}.\nServer state: {:?}.",
        req,
        client,
        server
    );

    let message;
    match req.parse::<messages::Message>() {
        Ok(m) => message = m,
        Err(ref e) => {
            warn!("Bad client message [{:?}]: {:?}.", req, e);
            return None;
        }
    };

    //TODO(lazau): Server prefix?
    match message.command {
        messages::Command::PASS => Some(req),
        messages::Command::NICK => {
            if message.params.len() == 0 {
                return Some("431 :No nickname given".to_string());
            }

            let nick = &message.params[0];
            // TODO(lazau): Validate nick based on
            // https://tools.ietf.org/html/rfc2812#section-2.3.1.
            {
                let mut server = server.lock().unwrap();
                if !server.nicknames.insert(nick.clone()) {
                    return Some(format!("433 {} :Nickname is already in use", nick));
                }
            }
            client.lock().unwrap().nick = Some(nick.clone());
            None
        }
        messages::Command::USER => {
            if message.params.len() < 4 {
                return Some("461 USER :Not enough parameters".to_string());
            }

            if message.params[1] != "0" || message.params[2] != "*" {
                warn!(
                    "USER command {:?}, second and third param unknown.",
                    message
                );
            }

            {
                let mut client = client.lock().unwrap();
                client.user = Some(UserData {
                    username: message.params[0].clone(),
                    realname: message.params[3].clone(),
                });

                if let Some(ref nick) = client.nick {
                    // TODO(lazau): <client>, <networkname> {:?}.
                    return Some(format!(
                        "001 <client> :Welcome to <networkname> Network, {}[!{}@{:?}]",
                        nick,
                        client.user.as_ref().unwrap().username,
                        client.remote_addr
                    ));
                }
            }
            None
        }
        u @ _ => {
            error!("Response to {:?} not yet implemented.", u);
            None
        }
    }
}
