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
use super::channel::{Message as ChannelMessage, Identifier as ChannelIdentifier, ChannelError,
                     ChannelTX};
use super::server::{Message as ServerMessage, ServerTX, ServerError};
use super::user::{User, Identifier as UserIdentifier, UserMode, SetMode};
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

    // Channel responses.
    ChannelJoin(
        Result<
            (ChannelIdentifier, Option<String>, Vec<UserIdentifier>, ChannelTX),
            (ChannelError, ChannelIdentifier),
        >
    ),

    // User responses.
    UserModeChanged(Arc<(user::Identifier, SetMode, Vec<UserMode>)>),

    // Server Responses.
    ServerRegistrationResult(Option<ServerError>),
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

macro_rules! send_log_err {
    ($tx:expr, $message:expr) => {
        match $tx.try_send($message) {
            Err(e) => debug!("Send error: {:?}.", e),
            _ =>{},
        };
    }
}

impl Connection {
    pub fn new(
        addr: SocketPair,
        shared_state: Arc<SharedState>,
        server_tx: ServerTX,
        tx: ConnectionTX,
    ) -> Self {
        // TODO(lazau): Resolve IP to fullname here.
        let hostname = addr.remote.ip().to_string();
        Connection {
            socket: addr,
            conn_type: ConnectionType::Registering(Registration::new(hostname)),
            shared_state: shared_state,
            server_tx: server_tx,
            tx: tx,
        }
    }

    fn get_registration(&self) -> &Registration {
        match self.conn_type {
            ConnectionType::Registering(ref r) => r,
            _ => unreachable!(),
        }
    }

    fn try_register(&mut self) {
        assert!(self.registered().is_none());
        let ident = if let ConnectionType::Registering(ref r) = self.conn_type {
            if r.nickname.is_none() || r.username.is_none() || r.realname.is_none() {
                return;
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

        send_log_err!(
            self.server_tx,
            ServerMessage::Register(ident.clone(), self.tx.clone())
        );
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
        let mut h;
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
                    self.try_register();
                    Vec::new()
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
                    self.try_register();
                    Vec::new()
                }
            }

            Command::JOIN(Requests::Join { join: jt }) => {
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
            Message::Join(ref user, ref channel) => {
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
            Message::Part(_, _, _) => unimplemented!(),
            /*result.push(Message {
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
            u @ _ => {
                error!("Broadcast message {:?} not yet implemented.", u);
                Vec::new()
            }
        }
    }
}

impl std::ops::Drop for Connection {
    fn drop(&mut self) {
        debug!("{:#?} disconnecting.", self.socket);
        if self.registered().is_some() {
            let ident = self.registered().unwrap().identifier().clone();
            send_log_err!(self.server_tx, ServerMessage::Disconnect(ident));
            // TODO(lazau): Part all channels.
        }
    }
}
