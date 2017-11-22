mod codec;
mod messages;

use hostname;
use chrono;

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
                .select(rx.then(|e| {
                    Ok(ClientEvent::Broadcast(
                        e.expect("client channel rx error. should never happen."),
                    ))
                }))
                .then(move |event| {
                    // ** Process future.
                    debug!("Connection event: {:?}.", event);
                    if event.is_err() {
                        let err = event.err().unwrap();
                        warn!("Upstream error: {:?}.", err);
                        return future::err(err);
                    }
                    let mut server_originated_messages: messages::MessageBuilder;
                    server_originated_messages.with_prefix(Some(hostname::get_hostname().expect(
                        "unable to get hostname",
                    )));

                    let res = match event.unwrap() {
                        ClientEvent::Socket(s) => {
                            process_message(
                                Arc::clone(&server_handle),
                                Arc::clone(&client_handle),
                                server_originated_messages,
                                s,
                            )
                        }
                        ClientEvent::Broadcast(_b) => {
                            unimplemented!("Unimplemented branch arm - broadcast message.");
                        }
                    };
                    future::ok(res)
                })
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
    mut message_builder: messages::MessageBuilder,
    req: String,
) -> Vec<messages::Message> {
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
            return Vec::new();
        }
    };

    match message.command {
        messages::Command::Req(ref r) => {
            match r {
                &messages::requests::Request::PASS => Vec::new(),//Some(req),
                &messages::requests::Request::NICK => {
                    if message.params.len() == 0 {
                        return vec![
                            message_builder
                                .with_command(messages::Command::Resp(
                                    messages::responses::Response::ERR_NONICKNAMEGIVEN,
                                ))
                                .build(),
                        ];
                    }

                    let nick = &message.params[0];
                    // TODO(lazau): Validate nick based on
                    // https://tools.ietf.org/html/rfc2812#section-2.3.1.
                    {
                        let mut server = server.lock().unwrap();
                        if !server.nicknames.insert(nick.clone()) {
                            return vec![
                                message_builder
                                    .with_command(messages::Command::Resp(
                                        messages::responses::Response::ERR_NICKNAMEINUSE,
                                    ))
                                    .build(),
                            ];
                        }
                        let mut client = client.lock().unwrap();
                        client.nick = Some(nick.clone());

                        match maybe_welcome_sequence(&server, &client, message_builder) {
                            Some(s) => s,
                            None => Vec::new(),
                        }
                    }
                }
                &messages::requests::Request::USER => {
                    if message.params.len() < 4 {
                        return vec![
                            message_builder
                                .with_command(messages::Command::Resp(
                                    messages::responses::Response::ERR_NEEDMOREPARAMS,
                                ))
                                .build(),
                        ];
                    }

                    if message.params[1] != "0" || message.params[2] != "*" {
                        warn!(
                            "USER command {:?}, second and third param unknown.",
                            message.command
                        );
                    }

                    {
                        let mut client = client.lock().unwrap();
                        client.user = Some(UserData {
                            username: message.params[0].clone(),
                            realname: message.params[3].clone(),
                        });

                        match maybe_welcome_sequence(
                            &*server.lock().unwrap(),
                            &client,
                            message_builder,
                        ) {
                            Some(s) => s,
                            None => Vec::new(),
                        }
                    }
                }
                u @ _ => {
                    error!("Response to {:?} not yet implemented.", u);
                    Vec::new()
                }
            }
        }
        _ => {
            warn!("{:?} isn't a client request. Dropping", message.command);
            Vec::new()
        }
    }
}

// Returns WELCOME sequence if client has successfully registered.
fn maybe_welcome_sequence(
    server: &Server,
    client: &ConnectionData,
    mut message_builder: messages::MessageBuilder,
) -> Option<Vec<messages::Message>> {
    if client.user.is_none() || client.nick.is_none() {
        return None;
    }

    Some(vec![
        message_builder
            .with_command(messages::Command::Resp(
                messages::responses::Response::RPL_WELCOME,
            ))
            .with_param_builder(Box::new(
                *messages::responses::WelcomeParamsBuilder::default()
            .with_nick(client.nick.as_ref().unwrap())
            // TODO(lazau): Network name from server configuration.
            .with_network_name(&"<network_name>".to_string())
            .with_user_and_host(&client.user.as_ref().unwrap().username, &client.remote_addr.to_string()),
            ))
            .build(),
        message_builder
            .with_command(messages::Command::Resp(messages::Response::RPL_YOURHOST))
            .with_params(vec![
                format!("{}", client.nick.as_ref().unwrap()),
                format!(
                    "Your host is {}, running version {}",
                    server.hostname,
                    server.version,
                ),
            ])
            .build(),
        message_builder
            .with_command(messages::Command::Resp(messages::Response::RPL_CREATED))
            .with_params(vec![
                format!("{}", client.nick.as_ref().unwrap()),
                format!(
                    "This server was created {}",
                    server.created.with_timezone(&chrono::offset::Local).to_rfc2822(),
                ),
            ])
            .build(),
        message_builder
            .with_command(messages::Command::Resp(messages::Response::RPL_MYINFO))
            .with_params(vec![
                format!("{}", client.nick.as_ref().unwrap()).to_string(),
            ])
            .build(),
    ])
}
