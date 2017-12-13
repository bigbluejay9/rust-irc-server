use futures::prelude::*;
use futures::*;
use futures::stream::*;
use futures::sink::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;
use std::{self, fmt, io};
use std::collections::HashMap;
use std::clone::Clone;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use super::{codec, user};
use super::messages::Message as IRCMessage;
use super::messages::commands::{Command, requests as Requests, responses as Responses};
use super::shared_state::SharedState;
use super::channel::{Identifier as ChannelIdentifier, ChannelError, Channel};
use super::server::{Server, Message as ServerMessage, ServerTX, ServerError};
use super::user::{User, UserTX, Message as UserMessage, Identifier as UserIdentifier, UserMode,
                  SetMode};
use super::super::templates;
use tokio_core;
use tokio_io::AsyncRead;

// Used to identify connections.
// Server is represented by (local, local) pair.
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct SocketPair {
    pub local: std::net::SocketAddr,
    pub remote: std::net::SocketAddr,
}

#[derive(Debug)]
enum ConnectionType {
    // Unkown type. After registration is complete will be either Client or Server.
    Registering(Registration),
    Client(UserIdentifier),
    Server, // unimplemented.
}

#[derive(Debug)]
pub enum Message {
    ServerRegistrationResult(Result<(UserIdentifier, UserTX), ServerError>),
}

pub type ConnectionTX = mpsc::Sender<Message>;

// A union of socket events and messages to the connection.
#[derive(Debug)]
enum ConnectionEvent {
    Socket(String),
    Message(Message),
}

#[derive(Debug)]
struct Registration {
    nickname: Option<String>,
    username: Option<String>,
    realname: Option<String>,
    hostname: String,
}

impl Registration {
    fn new(hostname: String) -> Self {
        Registration {
            nickname: None,
            username: None,
            realname: None,
            hostname,
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    // Unique per Connection.
    socket: SocketPair,
    conn_type: ConnectionType,
    shared_state: Arc<SharedState>,
}

impl fmt::Display for SocketPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({} : {})", self.local, self.remote)
    }
}

impl std::cmp::PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.socket == other.socket
    }
}

impl std::cmp::Eq for Connection {}

impl std::hash::Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.socket.hash(state);
    }
}

macro_rules! error_resp {
    ($err:expr) => { vec![ IRCMessage { prefix: None, command: $err } ] };
}

/*pub fn handle_new_connection(
    stream: tokio_core::net::TcpStream,
    shared_state: Arc<SharedState>,
    server_tx: ServerTX,
    thread_pool: CpuPool,
) {
    let socket = SocketPair {
        local: stream.local_addr().unwrap(),
        remote: stream.peer_addr().unwrap(),
    };
    let shared_state_serialization = Arc::clone(&shared_state);
    let (tx, rx) = mpsc::channel(shared_state.configuration.connection_message_queue_length);
    let multiplexed_sink = MultiplexedSink::new(server_tx.clone(), tx.clone());
    let connection = Arc::new(Mutex::new(Connection::new(
        socket.clone(),
        shared_state,
        Arc::clone(&multiplexed_sink.users),
        server_tx,
        tx,
    )));
    let c_stream = Arc::clone(&connection);
    let c_sink = Arc::clone(&connection);

    let (sink, stream) = stream.framed(codec::Utf8CrlfCodec).split();
    thread_pool
        .spawn_fn(move || {
            let connection = c_stream;
            stream
                .then(move |message| {
                    if message.is_err() {
                        error!("Cannot decode message: {:?}.", message.err());
                        return Ok(TXDestination::None);
                    }
                    // ** Process future.
                    let message = message.unwrap();
                    trace!("Incoming message: {:?}.", message);
                    let m = match message.parse::<IRCMessage>() {
                        Ok(m) => m,
                        // TODO(lazau): Maybe do some additional error processing here?
                        Err(e) => {
                            warn!("Failed to parse {}: {:?}.", message, e);
                            return Ok(TXDestination::None);
                        }
                    };
                    debug!("Request [{:?}].", m);
                    let mut connection = connection.lock().unwrap();
                    Ok(connection.process_irc_message(m))
                })
                .forward(multiplexed_sink)
                .then(move |_: Result<_, ()>| {
                    debug!("Connection {} input stream closed.", socket);
                    future::ok::<(), ()>(())
                })
        })
        .forget();
    thread_pool
        .spawn_fn(move || {
            let connection = c_sink;
            rx.then(move |connection_event| {
                let messages = match connection_event.expect("connection RX cannot fail") {
                    Message::Message(m) => m,
                    Message::Event(e) => connection.lock().unwrap().process_system_event(e),
                };
                let mut result: Vec<String> = Vec::new();
                // TODO(lazau): Perform 512 max line size here.
                for mut m in messages {
                    if m.prefix.is_none() {
                        m.prefix = Some(shared_state_serialization.hostname.clone());
                    }
                    result.push(format!("{}", m));
                }
                future::ok::<Vec<String>, io::Error>(result)
            }).forward(sink)
        })
        .forget();
}*/

impl Connection {
    pub fn handle_new_connection(
        stream: tokio_core::net::TcpStream,
        shared_state: Arc<SharedState>,
        server: Arc<Mutex<Server>>,
    ) -> Box<Future<Item = (), Error = io::Error>> {
        let socket = SocketPair {
            local: stream.local_addr().unwrap(),
            remote: stream.peer_addr().unwrap(),
        };
        let connection = Arc::new(Mutex::new(Connection::new(
            socket.clone(),
            shared_state.clone(),
            server.clone(),
        )));
        let (tx, rx) = mpsc::channel(shared_state.configuration.connection_message_queue_length);

        let connection_cleanup = Arc::clone(&connection);
        let shared_state_serialization = Arc::clone(&shared_state);

        let (sink, stream) = stream.framed(codec::Utf8CrlfCodec).split();
        let fut = stream
            .map(|m| ConnectionEvent::Socket(m))
            .select(rx.then(move |rx| {
                Ok(ConnectionEvent::Message(
                    rx.expect("connection RX cannot fail"),
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
                    ConnectionEvent::Socket(s) => {
                        let message = match s.parse::<IRCMessage>() {
                            Ok(m) => m,
                            // TODO(lazau): Maybe do some additional error processing here?
                            Err(e) => {
                                warn!("Failed to parse {}: {:?}.", s, e);
                                return future::ok(Vec::new());
                            }
                        };
                        connection.lock().unwrap().process_irc_message(message)
                    }
                    ConnectionEvent::Message(m) => {
                        connection.lock().unwrap().process_system_message(m)
                    }
                };
                future::ok(res)
            })
            .then(move |messages: Result<Vec<IRCMessage>, _>| {
                // ** Serialization future.
                if messages.is_err() {
                    return future::err(messages.err().unwrap());
                }
                let mut result = Vec::new();
                // TODO(lazau): Perform 512 max line size here.
                for mut m in messages.unwrap() {
                    if m.prefix.is_none() {
                        m.prefix = Some(shared_state_serialization.hostname.clone());
                    }
                    // TODO(lazau): Convert serialization error to future::err.
                    result.push(format!("{}", m));
                }
                future::ok(result)
            })
            .forward(sink)
            .then(move |e: Result<(_, _), io::Error>| {
                // ** Cleanup future.
                let client = connection_cleanup.lock().unwrap().disconnect();
                if let Err(e) = e {
                    warn!("Connection error: {:?}.", e);
                }
                Ok(())
            });
        Box::new(fut)

        /*.then(move |event| {
                trace!("Connection event {:?}.", event);
                if event.is_err() {
                    error!("Cannot decode message: {:?}.", message.err());
                    return None;
                }
                // ** Process future.
                let message = message.unwrap();
                trace!("Incoming message: {:?}.", message);
                let m = match message.parse::<IRCMessage>() {
                    Ok(m) => m,
                    // TODO(lazau): Maybe do some additional error processing here?
                    Err(e) => {
                        warn!("Failed to parse {}: {:?}.", message, e);
                        return None;
                    }
                };
                debug!("Request [{:?}].", m);
                let mut connection = connection.lock().unwrap();
                connection.process_irc_message(m)
            })
            .then(move |message| {
                let messages = match connection_event.expect("connection RX cannot fail") {
                    Message::Message(m) => m,
                    Message::Event(e) => connection.lock().unwrap().process_system_event(e),
                };
                let mut result: Vec<String> = Vec::new();
                // TODO(lazau): Perform 512 max line size here.
                for mut m in messages {
                    if m.prefix.is_none() {
                        m.prefix = Some(shared_state_serialization.hostname.clone());
                    }
                    result.push(format!("{}", m));
                }
                future::ok::<Vec<String>, io::Error>(result)
            })
            .forward(multiplexed_sink)*/

    }

    fn new(addr: SocketPair, shared_state: Arc<SharedState>, server: Arc<Mutex<Server>>) -> Self {
        // TODO(lazau): Resolve IP to fullname here.
        let hostname = addr.remote.ip().to_string();
        Connection {
            socket: addr,
            conn_type: ConnectionType::Registering(Registration::new(hostname)),
            shared_state: shared_state,
        }
    }

    fn get_registration(&self) -> &Registration {
        match self.conn_type {
            ConnectionType::Registering(ref r) => r,
            _ => unreachable!(),
        }
    }

    fn try_register(&mut self) -> Vec<IRCMessage> {
        assert!(self.registered().is_none());
        let ident = if let ConnectionType::Registering(ref r) = self.conn_type {
            if r.nickname.is_none() || r.username.is_none() || r.realname.is_none() {
                return Vec::new();
            }
            UserIdentifier {
                nickname: r.nickname.as_ref().unwrap().clone(),
                username: r.username.as_ref().unwrap().clone(),
                realname: r.realname.as_ref().unwrap().clone(),
                hostname: self.socket.remote.ip().to_string(),
            }
        } else {
            unreachable!()
        };

        /*TXDestination::Server(vec![
            ServerMessage::Register(ident.clone(), self.tx.clone()),
        ])*/
        Vec::new()
    }

    fn add_registration_info(
        &mut self,
        nickname: Option<String>,
        username: Option<String>,
        realname: Option<String>,
    ) {
        let mut n;
        let mut u;
        let mut r;
        let h;
        match self.conn_type {
            ConnectionType::Registering(Registration {
                                            ref nickname,
                                            ref username,
                                            ref realname,
                                            ref hostname,
                                        }) => {
                n = nickname.clone();
                u = username.clone();
                r = realname.clone();
                h = hostname.clone();
            }
            _ => unreachable!(),
        };

        if nickname.is_some() {
            n = nickname;
        }
        if username.is_some() {
            u = username;
        }
        if realname.is_some() {
            r = realname;
        }

        self.conn_type = ConnectionType::Registering(Registration {
            nickname: n,
            username: u,
            realname: r,
            hostname: h,
        });
    }

    fn registered(&self) -> Option<&UserIdentifier> {
        match self.conn_type {
            ConnectionType::Registering(_) => None,
            ConnectionType::Client(ref u) => Some(u),
            ConnectionType::Server => unimplemented!(),
        }
    }

    pub fn process_irc_message(&mut self, req: IRCMessage) -> Vec<IRCMessage> {
        trace!("Connection state: {:?}.", self);

        macro_rules! verify_registered {
            () => {
                if self.registered().is_none() {
                    return error_resp!(Command::ERR_NOTREGISTERED(Responses::NOTREGISTERED::default()));
                }
            }
        }

        match req.command {
            Command::NICK(Requests::Nick { nickname: nick }) => {
                // TODO(lazau): Validate nick based on
                // https://tools.ietf.org/html/rfc2812#section-2.3.1.
                if self.registered().is_some() {
                    unimplemented!()
                } else {
                    self.add_registration_info(Some(nick.clone()), None, None);
                    self.try_register()
                }
            }

            Command::USER(Requests::User {
                              username,
                              mode: _mode,
                              unused: _unused,
                              realname,
                          }) => {
                if self.registered().is_some() {
                    let user = self.registered().unwrap();
                    return error_resp!(Command::ERR_ALREADYREGISTRED(
                        Responses::AlreadyRegistered { nick: user.nickname.clone() },
                    ));
                } else {
                    self.add_registration_info(None, Some(username), Some(realname));
                    self.try_register()
                }
            }

            u @ _ => {
                error!("{:?} not yet implemented.", u);
                Vec::new()
                //TXDestination::None
            }
        }
    }

    pub fn process_system_message(&mut self, m: Message) -> Vec<IRCMessage> {
        match m {
            _ => unimplemented!(),
            /*Event::ServerRegistrationResult(r) => {
                match r {
                    Ok((ident, tx)) => {
                        let nickname = ident.nickname.clone();
                        MultiplexedSink::add_user(&self.sink_map, &ident, tx);
                        self.conn_type = ConnectionType::Client(ident);
                        vec![
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_WELCOME(Responses::Welcome {
                                    nick: nickname.clone(),
                                    message: self.shared_state
                                        .template_engine
                                        .0
                                        .render(
                                            templates::RPL_WELCOME_TEMPLATE_NAME,
                                            &templates::Welcome {
                                                network_name: &self.shared_state
                                                    .configuration
                                                    .network_name,
                                                nick: &nickname,
                                            },
                                        )
                                        .unwrap(),
                                }),
                            },
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_YOURHOST(Responses::YourHost {
                                    nick: nickname.clone(),
                                    message: self.shared_state
                                        .template_engine
                                        .0
                                        .render(
                                            templates::RPL_YOURHOST_TEMPLATE_NAME,
                                            &templates::YourHost {
                                                hostname: &self.shared_state.hostname,
                                                version: &self.shared_state.configuration.version,
                                            },
                                        )
                                        .unwrap(),
                                }),
                            },
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_CREATED(Responses::Created {
                                    nick: nickname.clone(),
                                    message: self.shared_state
                                        .template_engine
                                        .0
                                        .render(
                                            templates::RPL_CREATED_TEMPLATE_NAME,
                                            &templates::Created {
                                                created: &format!(
                                                    "{:?}",
                                                    &self.shared_state.created
                                                ),
                                            },
                                        )
                                        .unwrap(),
                                }),
                            },
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_MYINFO(Responses::MyInfo::default()),
                            },
                        ]
                    }
                    Err(server_error) => {
                        match server_error {
                            ServerError::NickInUse => {
                                error_resp!(Command::ERR_NICKNAMEINUSE(Responses::NICKNAMEINUSE {
                                    nick: self.get_registration()
                                        .nickname
                                        .as_ref()
                                        .unwrap()
                                        .clone(),
                                }))
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }*/
        }
    }

    /*Command::JOIN(Requests::Join { join: jt }) => {
                verify_registered!();
                match jt {
                    Requests::JoinChannels::PartAll => {

                        let user = self.registered_mut().unwrap();
                        /*connection.server.lock().unwrap().part_all(
                                user.identifier(),
                                None,
                                );*/
                        unimplemented!()
                    }
                    Requests::JoinChannels::Channels(r) => {
                        let ident = self.registered().unwrap().identifier().clone();
                        for chan in r {
                            let msg = ServerMessage::Join(
                                ident.clone(),
                                self.tx.clone(),
                                ChannelIdentifier::from_name(&chan),
                                None,
                            );
                            send_log_err!(self.server_tx, msg);
                        }
                    }
                    Requests::JoinChannels::KeyedChannels(r) => {
                        unimplemented!()
                        /*let (chans, keys): (Vec<String>, Vec<String>) = r.into_iter().unzip();
                        connection.server.lock().unwrap().join(
                                user.identifier(),
                                chans
                                .iter()
                                .zip(keys.iter().map(|k| Some(k)))
                                .collect(),
                                );*/
                    }
                };
                Vec::new()
            }

            Command::PART(Requests::Part { channels, message }) => {
                unimplemented!()
                /*let mut connection = connection.lock().unwrap();
                let user = verify_registered_and_return_user!();
                connection.server.lock().unwrap().part(
                        user.identifier(),
                        &channels,
                        &message,
                        );
                Vec::new()*/
            }

            Command::MODE(Requests::Mode {
                              target,
                              mode_string,
                              mode_args,
                          }) => {
                verify_registered!();
                // MODE query.
                if mode_string.is_none() {
                    unimplemented!();
                }

                // MODE adjustment.
                let user = self.registered_mut().unwrap();
                if &target != user.nick() {
                    return_error_resp!(Command::ERR_USERSDONTMATCH(
                        Responses::UsersDontMatch { nick: user.nick().clone() },
                    ));
                }
                let mode = mode_string.unwrap();
                if mode.len() < 2 {
                    return_error_resp!(Command::ERR_UMODEUNKNOWNFLAG(
                        Responses::UModeUnknownFlag { nick: user.nick().clone() },
                    ));
                }
                let set = if mode.starts_with("+") {
                    SetMode::Add
                } else if mode.starts_with("-") {
                    SetMode::Remove
                } else {
                    return_error_resp!(Command::ERR_UMODEUNKNOWNFLAG(
                        Responses::UModeUnknownFlag { nick: user.nick().clone() },
                    ));
                };

                let mut modes = Vec::new();
                for c in mode.chars().skip(1) {
                    match c.to_string().parse::<UserMode>() {
                        Err(e) => {
                            return_error_resp!(Command::ERR_UMODEUNKNOWNFLAG(
                                Responses::UModeUnknownFlag { nick: user.nick().clone() },
                            ));
                        }
                        Ok(m) => modes.push(m),
                    };
                }

                user.set_mode(&set, &modes);
                Vec::new()

            }

            Command::PING(Requests::Ping { originator, target }) => {
                if target.is_some() && target.unwrap() != self.shared_state.hostname {
                    // Forward to another server.
                    unimplemented!();
                }
                vec![
                    IRCMessage {
                        prefix: None,
                        command: Command::PONG(Requests::Pong {
                            originator: self.shared_state.hostname.clone(),
                            target: None,
                        }),
                    },
                ]
            }

            u @ _ => {
                error!("Response to {:?} not yet implemented.", u);
                Vec::new()
            }
        }
    }

    pub fn process_system_message(&mut self, message: Message) -> Vec<IRCMessage> {
        trace!("Connection state: {:?}.", self);
        match message {
            /*Message::Join(ref user, ref channel) => {
                unimplemented!()
                vec![
                Message {
prefix: Some(format!("{}", user)),
            command: Command::JOIN(Requests::Join {
join: Requests::JoinChannels::Channels(vec![channel.clone()]),
}),
            } if chan.is_err() {
                warn!("Failed to join {}: {:?}.", c, chan.err().unwrap());
                continue;
                }
                let chan = chan.unwrap();
                self.channels.insert(chan.name.clone());
                result.push(Message {
prefix: None,
command: Command::JOIN(Requests::Join {
join: Requests::JoinChannels::Channels(vec![c.clone()]),
}),
});
result.push(Message {
prefix: None,
command: Command::RPL_TOPIC(Responses::Topic {
nick: self.nick.as_ref().unwrap().clone(),
channel: c.clone(),
topic: chan.topic.clone(),
}),
});
result.push(Message {
prefix: None,
command: Command::RPL_NAMREPLY(Responses::NamReply {
nick: self.nick.as_ref().unwrap().clone(),
// XXX
symbol: "".to_string(),
channel: c.clone(),
// xxx
members: chan.nicks
.iter()
.cloned()
.map(|n| ("".to_string(), n))
.collect(),
}),
});,
]
            }
            Message::Part(_, _, _) => unimplemented!(),
            result.push(Message {
      prefix: None,
      command: Command::PART(Requests::Part {
      channels: vec![c.clone()],
      message: None,
      }),
      });*/
            Message::PrivateMessage => unimplemented!(),

            Message::ChannelJoin(r) => {
                match r {
                    Ok((ident, topic, members, tx)) => {
                        let user = self.registered_mut().unwrap();
                        user.joined_channel(&ident, tx);
                        vec![
                        // TODO(lazau): XXX 
                        ]
                    }
                    Err((e, ident)) => {
                        let user = self.registered().unwrap();
                        match e {
                            ChannelError::BadKey => {
                                return_error_resp!(
                                    Command::ERR_BADCHANNELKEY(Responses::BadChannelKey {
                                        nick: user.nick().clone(),
                                        channel: ident.name,
                                    })
                                );
                            }
                            ChannelError::Banned => {
                                return_error_resp!(
                                    Command::ERR_BANNEDFROMCHAN(Responses::BannedFromChan {
                                        nick: user.nick().clone(),
                                        channel: ident.name,
                                    })
                                );
                            }
                            ChannelError::AlreadyMember => {
                                warn!(
                                    "{:?} trying to join a channel I'm already a member of.",
                                    self
                                );
                                Vec::new()
                            }
                        }
                    }
                }
            }

            Message::UserModeChanged(a) => {
                let &(ref ident, ref set, ref modified_modes) = a.deref();
                let mut mode_string_response: String = match set {
                    &SetMode::Add => "+".to_string(),
                    &SetMode::Remove => "-".to_string(),
                };
                for m in modified_modes {
                    mode_string_response.push_str(&format!("{}", m));
                }
                vec![
                    IRCMessage {
                        prefix: Some(ident.as_prefix()),
                        command: Command::MODE(Requests::Mode {
                            target: ident.nickname.clone(),
                            mode_string: Some(mode_string_response),
                            mode_args: None,
                        }),
                    },
                ]
            }

            Message::ServerRegistrationResult(se) => {
                match se {
                    None => {
                        let nickname;
                        let username;
                        let realname;
                        let hostname;
                        {
                            let r = self.get_registration();
                            nickname = r.nickname.as_ref().unwrap().clone();
                            username = r.username.as_ref().unwrap().clone();
                            realname = r.realname.as_ref().unwrap().clone();
                            hostname = r.hostname.clone();

                        }
                        self.conn_type = ConnectionType::Client(User::new(
                            &nickname,
                            &username,
                            &realname,
                            &hostname,
                            self.tx.clone(),
                        ));

                        return vec![
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_WELCOME(Responses::Welcome {
                                    nick: nickname.clone(),
                                    message: self.shared_state
                                        .template_engine
                                        .0
                                        .render(
                                            templates::RPL_WELCOME_TEMPLATE_NAME,
                                            &templates::Welcome {
                                                network_name: &self.shared_state
                                                    .configuration
                                                    .network_name,
                                                nick: &nickname,
                                            },
                                        )
                                        .unwrap(),
                                }),
                            },
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_YOURHOST(Responses::YourHost {
                                    nick: nickname.clone(),
                                    message: self.shared_state
                                        .template_engine
                                        .0
                                        .render(
                                            templates::RPL_YOURHOST_TEMPLATE_NAME,
                                            &templates::YourHost {
                                                hostname: &self.shared_state.hostname,
                                                version: &self.shared_state.configuration.version,
                                            },
                                        )
                                        .unwrap(),
                                }),
                            },
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_CREATED(Responses::Created {
                                    nick: nickname.clone(),
                                    message: self.shared_state
                                        .template_engine
                                        .0
                                        .render(
                                            templates::RPL_CREATED_TEMPLATE_NAME,
                                            &templates::Created {
                                                created: &format!(
                                                    "{:?}",
                                                    &self.shared_state.created
                                                ),
                                            },
                                        )
                                        .unwrap(),
                                }),
                            },
                            IRCMessage {
                                prefix: None,
                                command: Command::RPL_MYINFO(Responses::MyInfo::default()),
                            },
                        ];
                    }
                    Some(err) => {
                        match err {
                            ServerError::NickInUse => {
                                return_error_resp!(
                                    Command::ERR_NICKNAMEINUSE(Responses::NICKNAMEINUSE {
                                        nick: self.get_registration()
                                            .nickname
                                            .as_ref()
                                            .unwrap()
                                            .clone(),
                                    })
                                );
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            
        }
    }*/

    fn disconnect(&mut self) {
        debug!("{:#?} disconnecting.", self.socket);
        if self.registered().is_some() {
            /*let ident = self.registered().unwrap().identifier().clone();
            send_log_err!(self.server_tx, ServerMessage::Disconnect(ident));*/
            unimplemented!()
            // TODO(lazau): Part all channels.
        }
    }
}
