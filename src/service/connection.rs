use futures::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;
use std::{self, fmt, io};
use std::clone::Clone;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use super::{codec, user};
use super::messages::Message as IRCMessage;
use super::messages::commands::{Command, requests as Requests, responses as Responses};
use super::shared_state::SharedState;
use super::server::{ServerTX, ServerError};
use super::user::{User, Identifier as UserIdentifier};
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
    Client(User),
    Server, // unimplemented.
}

#[derive(Debug)]
pub enum Message {
    // (User, Channel).
    Join(user::Identifier, String),
    // (User, Channel, Message)
    Part(user::Identifier, String, Option<String>),
    PrivateMessage,

    ServerRegistrationResult(Option<ServerError>),
}

pub type ConnectionTX = mpsc::Sender<Arc<Message>>;

// A union of socket events and messages to the connection.
#[derive(Debug)]
enum ConnectionEvent {
    Socket(String),
    Message(Arc<Message>),
}

#[derive(Debug, Default)]
struct Registration {
    nickname: Option<String>,
    username: Option<String>,
    realname: Option<String>,
}

#[derive(Debug)]
pub struct Connection {
    // Unique per Connection.
    socket: SocketPair,
    shared_state: Arc<SharedState>,
    conn_type: ConnectionType,
    server_tx: ServerTX,
    tx: ConnectionTX,
}

impl fmt::Display for SocketPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({} : {})", self.local, self.remote)
    }
}

impl std::default::Default for ConnectionType {
    fn default() -> Self {
        ConnectionType::Registering(Registration::default())
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


macro_rules! return_error_resp {
        ($err:expr) => {
            return vec![
                IRCMessage {
prefix: None,
            command: $err,
                },
            ];
        };
    }

pub fn handle_new_connection(
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
    // TODO(lazau): How large should this buffer be?
    // Note: should penalize slow connections.
    let (tx, rx) = mpsc::channel(shared_state.configuration.connection_message_queue_length);
    let connection = Arc::new(Mutex::new(
        Connection::new(socket, shared_state, server_tx, tx),
    ));

    let (sink, stream) = stream.framed(codec::Utf8CrlfCodec).split();
    thread_pool
        .spawn_fn(move || {
            stream
                .map(|s| ConnectionEvent::Socket(s))
                .select(rx.then(|e| Ok(ConnectionEvent::Message(e.unwrap()))))
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
                            debug!("Request [{:?}].", message);
                            connection.lock().unwrap().process_irc_message(message)
                        }

                        ConnectionEvent::Message(m) => {
                            debug!("Message [{:?}].", m);
                            connection.lock().unwrap().process_system_message(m)
                        }
                    };
                    debug!("Response [{:?}].", res);
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
                .then(
                    move |e: Result<(_, _), io::Error>| -> future::FutureResult<(), ()> {
                        // ** Cleanup future.
                        if let Err(e) = e {
                            warn!("Connection error: {:?}.", e);
                        }
                        future::ok(())
                    },
                    // Connection gets dropped.
                )
        })
        .forget();
}

impl Connection {
    pub fn new(
        addr: SocketPair,
        shared_state: Arc<SharedState>,
        server_tx: ServerTX,
        tx: ConnectionTX,
    ) -> Self {
        Connection {
            socket: addr,
            conn_type: ConnectionType::default(),
            shared_state: shared_state,
            server_tx: server_tx,
            tx: tx,
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
            }
        } else {
            unreachable!()
        };

        unimplemented!()
        /*match self.server.add_user(&ident, self.tx.clone()) {
            Ok(_) => {
                self.conn_type = ConnectionType::Client(User::new(
                    ident.nickname.clone(),
                    ident.username.clone(),
                    ident.realname.clone(),
                ));
                let nick = ident.nickname;

                #[derive(Serialize)]
                struct WelcomeData<'a> {
                    network_name: &'a str,
                    nick: &'a str,
                }

                return vec![
                    IRCMessage {
                        prefix: None,
                        command: Command::RPL_WELCOME(Responses::Welcome {
                            nick: nick.clone(),
                            message: self.shared_state
                                .template_engine
                                .0
                                .render(
                                    templates::RPL_WELCOME_TEMPLATE_NAME,
                                    &WelcomeData {
                                        network_name: &self.shared_state.configuration.network_name,
                                        nick: &nick,
                                    },
                                )
                                .unwrap(),
                        }),
                    },
                    IRCMessage {
                        prefix: None,
                        command: Command::RPL_YOURHOST(Responses::YourHost {
                            nick: nick.clone(),
                            message: self.shared_state
                                .template_engine
                                .0
                                .render(
                                    templates::RPL_YOURHOST_TEMPLATE_NAME,
                                    self.shared_state.deref(),
                                )
                                .unwrap(),
                        }),
                    },
                    IRCMessage {
                        prefix: None,
                        command: Command::RPL_CREATED(Responses::Created {
                            nick: nick.clone(),
                            message: self.shared_state
                                .template_engine
                                .0
                                .render(
                                    templates::RPL_CREATED_TEMPLATE_NAME,
                                    self.shared_state.deref(),
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
            Err(ServerError::NickInUse) => {
                return_error_resp!(Command::ERR_NICKNAMEINUSE(
                    Responses::NICKNAMEINUSE { nick: ident.nickname.clone() },
                ));
            }
            _ => unreachable!(),
        }*/
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
        match self.conn_type {
            ConnectionType::Registering(Registration {
                                            ref nickname,
                                            ref username,
                                            ref realname,
                                        }) => {
                n = nickname.clone();
                u = username.clone();
                r = realname.clone();
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
        });
    }

    fn registered(&self) -> Option<&User> {
        match self.conn_type {
            ConnectionType::Registering(_) => None,
            ConnectionType::Client(ref u) => Some(u),
            ConnectionType::Server => unimplemented!(),
        }
    }

    fn registered_mut(&mut self) -> Option<&mut User> {
        match self.conn_type {
            ConnectionType::Registering(_) => None,
            ConnectionType::Client(ref mut u) => Some(u),
            ConnectionType::Server => unimplemented!(),
        }
    }

    pub fn disconnect(&mut self) {
        //TODO(lazau): Maybe gotta part all channels?
        unimplemented!()
    }

    pub fn process_irc_message(&mut self, req: IRCMessage) -> Vec<IRCMessage> {
        trace!("Connection state: {:?}.", self);

        macro_rules! verify_registered_and_return_user {
            () => {
                if let Some(u) = self.registered() {
                    u
                } else {
                    return_error_resp!(Command::ERR_NOTREGISTERED(Responses::NOTREGISTERED::default()));
                }
            }
        }

        macro_rules! verify_registered {
            () => {
                if self.registered().is_none() { 
                    return_error_resp!(Command::ERR_NOTREGISTERED(Responses::NOTREGISTERED::default()));
                }
            }
        }

        match req.command {
            Command::NICK(Requests::Nick { nickname: nick }) => {
                // TODO(lazau): Validate nick based on
                // https://tools.ietf.org/html/rfc2812#section-2.3.1.
                if self.registered().is_some() {
                    let user = self.registered().unwrap();
                    let mut new_ident = user.identifier().clone();
                    new_ident.nickname = nick.clone();
                    unimplemented!()
                /*match self.server.replace_nick(user.identifier(), &new_ident) {
                        Err(ServerError::NickInUse) => {
                            return_error_resp!(Command::ERR_NICKNAMEINUSE(
                                Responses::NICKNAMEINUSE { nick: nick.clone() },
                            ));
                        }
                        // NICK change.
                        Ok(_) => unimplemented!(),
                        _ => unreachable!(),
                    }*/
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
                    return_error_resp!(Command::ERR_ALREADYREGISTRED(
                        Responses::AlreadyRegistered { nick: user.nick().clone() },
                    ));
                } else {
                    self.add_registration_info(None, Some(username), Some(realname));
                    self.try_register()
                }
            }

            Command::JOIN(Requests::Join { join: jt }) => {
                //let mut user = verify_registered_and_return_user!();
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
                        unimplemented!()
                        /*connection.server.lock().unwrap().join(
                                user.identifier(),
                                r.iter().zip(iter::repeat(None)).collect(),
                                );*/
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
                /*let mut conn = connection.lock().unwrap();
                let user = verify_registered_and_return_user!();
                if &target != user.nick() {
                    return_error_resp!(Command::ERR_USERSDONTMATCH(
                                Responses::UsersDontMatch { nick: user.nick().clone() },
                                ));
                }
                if mode_string.is_none() {
                    unimplemented!();
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

                let m = mode.chars()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .parse::<UserMode>()
                    .map_err(|_| {
                            return_error_resp!(Command::ERR_UMODEUNKNOWNFLAG(
                                        Responses::UModeUnknownFlag { nick: user.nick().clone() },
                                        ))
                            })
                .unwrap();
                user.set_mode(&set, &vec![m]);
                Vec::new()*/
                unimplemented!()
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

    pub fn process_system_message(&mut self, message: Arc<Message>) -> Vec<IRCMessage> {
        trace!("Connection state: {:?}.", self);
        match message.deref() {
            &Message::Join(ref user, ref channel) => {
                unimplemented!()
                /*vec![
                Message {
prefix: Some(format!("{}", user)),
            command: Command::JOIN(Requests::Join {
join: Requests::JoinChannels::Channels(vec![channel.clone()]),
}),
            } /*if chan.is_err() {
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
});*/,
]*/
            }
            &Message::Part(_, _, _) => unimplemented!(),
            /*result.push(Message {
      prefix: None,
      command: Command::PART(Requests::Part {
      channels: vec![c.clone()],
      message: None,
      }),
      });*/
            &Message::PrivateMessage => unimplemented!(),
            u @ _ => {
                error!("Broadcast message {:?} not yet implemented.", u);
                Vec::new()
            }
        }
    }
}

impl std::ops::Drop for Connection {
    fn drop(&mut self) {
        debug!("{:#?} disconnecting.", self);
    }
}
