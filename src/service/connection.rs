use futures::sync::mpsc;

use std::{self, iter, str};
use std::clone::Clone;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use super::{SocketPair, Broadcast};
use super::messages::Message;
use super::messages::commands::Command;
use super::messages::commands::requests as Requests;
use super::messages::commands::responses as Responses;
use super::server::{Configuration, Server, ServerError};
use super::templates;
use super::user::{SetMode, User, UserMode, Identifier as UserIdentifier};

#[derive(Debug)]
enum ConnectionType {
    // Unkown type. After registration is complete will be either Client or Server.
    Registering(Registration),

    Client(User),
    // TODO(lazau): To support server connections, use something like HashSet<User>.
    Server, // unimplemented.
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
    pub socket: SocketPair,

    conn_type: ConnectionType,

    // Implicity enforce locking order by only allowing Server access through Connection.
    // (thereby ensuring that the Connection lock is held before the server).
    pub server: Arc<Mutex<Server>>,

    tx: mpsc::Sender<Arc<Broadcast>>,
}

impl std::cmp::PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.socket == other.socket
    }
}

impl std::hash::Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.socket.hash(state);
    }
}

impl Connection {
    pub fn new(
        addr: SocketPair,
        tx: mpsc::Sender<Arc<Broadcast>>,
        server: Arc<Mutex<Server>>,
    ) -> Self {
        Connection {
            socket: addr,
            conn_type: ConnectionType::Registering(Registration::default()),
            server: server,
            tx: tx,
        }
    }

    pub fn get_user(&self) -> &User {
        match self.conn_type {
            ConnectionType::Client(ref u) => u,
            _ => panic!("Trying to get user for bad connection type."),
        }
    }

    pub fn get_unregistered_nick(&self) -> String {
        match self.conn_type {
            ConnectionType::Registering(Registration { nickname, .. }) => nickname.unwrap().clone(),
            _ => panic!("Trying to get unregistered nick for registered connection."),
        }
    }

    pub fn try_register(&mut self) -> bool {
        assert!(self.ready_to_register());

        if let ConnectionType::Registering(Registration {
                                               nickname,
                                               username,
                                               realname,
                                           }) = self.conn_type
        {
            let ident = &UserIdentifier {
                nickname: nickname.unwrap(),
                username: username.unwrap(),
                realname: realname.unwrap(),
            };
            if let Err(_) = self.server.lock().unwrap().add_user(ident, self.tx.clone()) {
                return false;
            } else {
                self.conn_type = ConnectionType::Client(User::new(
                    nickname.unwrap().clone(),
                    username.unwrap().clone(),
                    realname.unwrap().clone(),
                ));
            }
        } else {
            assert!(false, "trying to register when not yet ready.");
        }
        true
    }

    pub fn ready_to_register(&self) -> bool {
        if let ConnectionType::Registering(ref r) = self.conn_type {
            if r.nickname.is_some() || r.username.is_some() || r.realname.is_some() {
                return true;
            }
        }
        false
    }

    pub fn add_registration_info(
        &mut self,
        nickname: Option<String>,
        username: Option<String>,
        realname: Option<String>,
    ) {
        let n;
        let u;
        let r;
        match self.conn_type {
            ConnectionType::Registering(Registration {
                                            ref nickname,
                                            ref username,
                                            ref realname,
                                        }) => {
                n = nickname;
                u = username;
                r = realname;
            }
            _ => unreachable!(),
        };

        if nickname.is_some() {
            n = &nickname;
        }
        if username.is_some() {
            u = &username;
        }
        if realname.is_some() {
            r = &realname;
        }

        self.conn_type = ConnectionType::Registering(Registration {
            nickname: n.clone(),
            username: u.clone(),
            realname: r.clone(),
        });
    }

    pub fn registered(&self) -> Option<&User> {
        match self.conn_type {
            ConnectionType::Registering(_) => None,
            ConnectionType::Client(ref u) => Some(u),
            ConnectionType::Server => unimplemented!(),
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(ref u) = self.registered() {
            self.server.lock().unwrap().replace_user(
                u.identifier(),
                None,
            );
        } else {
            return;
        }
        //TODO(lazau): Maybe gotta part all channels?
    }
}

macro_rules! error_resp {
    ($err:expr) => {
        return vec![
            Message {
                prefix: None,
                command: $err,
            },
        ];
    };
}

pub fn process_message(
    configuration: Arc<Configuration>,
    connection: Arc<Mutex<Connection>>,
    req: Message,
) -> Vec<Message> {
    trace!("Connection state: {:?}.", connection);

    macro_rules! verify_registered_and_return_user {
        ($connection:expr) => {
            if let Some(ref u) = $connection.registered() {
                u
            } else {
                error_resp!(Command::ERR_NOTREGISTERED(Responses::NOTREGISTERED::default()));
            }
        }
    }

    match req.command {
        Command::NICK(Requests::Nick { nickname: nick }) => {
            // TODO(lazau): Validate nick based on
            // https://tools.ietf.org/html/rfc2812#section-2.3.1.
            let mut conn = connection.lock().unwrap();
            let registered = conn.registered();
            if let Some(user) = registered {
                let mut new_ident = user.identifier().clone();
                new_ident.nickname = nick.clone();
                if conn.server
                    .lock()
                    .unwrap()
                    .replace_user(&user.identifier(), Some(&new_ident))
                    .err() == Some(ServerError::NickInUse)
                {
                    error_resp!(Command::ERR_NICKNAMEINUSE(
                        Responses::NICKNAMEINUSE { nick: nick.clone() },
                    ));
                }
                // NICK change.
                unimplemented!();
            } else {
                conn.add_registration_info(Some(nick.clone()), None, None);
                if conn.ready_to_register() {
                    if !conn.try_register() {
                        error_resp!(Command::ERR_NICKNAMEINUSE(
                            Responses::NICKNAMEINUSE { nick: nick.clone() },
                        ));
                    } else {
                        welcome_sequence(&configuration, conn.get_user())
                    }
                } else {
                    Vec::new()
                }
            }
        }

        Command::USER(Requests::User {
                          username,
                          mode: _mode,
                          unused: _unused,
                          realname,
                      }) => {
            let mut conn = connection.lock().unwrap();
            if let Some(ref u) = conn.registered() {
                error_resp!(Command::ERR_ALREADYREGISTRED(
                    Responses::AlreadyRegistered { nick: u.nick().clone() },
                ));
            } else {
                conn.add_registration_info(None, Some(username), Some(realname));
                if conn.ready_to_register() {
                    if !conn.try_register() {
                        error_resp!(Command::ERR_NICKNAMEINUSE(Responses::NICKNAMEINUSE {
                            nick: conn.get_unregistered_nick(),
                        }));
                    } else {
                        welcome_sequence(&configuration, conn.get_user())
                    }
                } else {
                    Vec::new()
                }
            }
        }

        Command::JOIN(Requests::Join { join: jt }) => {
            let mut connection = connection.lock().unwrap();
            let user = verify_registered_and_return_user!(connection);
            match jt {
                Requests::JoinChannels::PartAll => {
                    connection.server.lock().unwrap().part_all(
                        user.identifier(),
                        None,
                    );
                }
                Requests::JoinChannels::Channels(r) => {
                    connection.server.lock().unwrap().join(
                        user.identifier(),
                        r.iter().zip(iter::repeat(None)).collect(),
                    );
                }
                Requests::JoinChannels::KeyedChannels(r) => {
                    let (chans, keys): (Vec<String>, Vec<String>) = r.into_iter().unzip();
                    connection.server.lock().unwrap().join(
                        user.identifier(),
                        chans
                            .iter()
                            .zip(keys.iter().map(|k| Some(k)))
                            .collect(),
                    );
                }
            };
            Vec::new()
        }

        Command::PART(Requests::Part { channels, message }) => {
            let mut connection = connection.lock().unwrap();
            let user = verify_registered_and_return_user!(connection);
            connection.server.lock().unwrap().part(
                user.identifier(),
                &channels,
                &message,
            );
            Vec::new()
        }

        Command::MODE(Requests::Mode {
                          target,
                          mode_string,
                          mode_args,
                      }) => {
            let mut conn = connection.lock().unwrap();
            let user = verify_registered_and_return_user!(conn);
            if &target != user.nick() {
                error_resp!(Command::ERR_USERSDONTMATCH(
                    Responses::UsersDontMatch { nick: user.nick().clone() },
                ));
            }
            if mode_string.is_none() {
                unimplemented!();
            }
            let mode = mode_string.unwrap();
            if mode.len() < 2 {
                error_resp!(Command::ERR_UMODEUNKNOWNFLAG(
                    Responses::UModeUnknownFlag { nick: user.nick().clone() },
                ));
            }
            let set = if mode.starts_with("+") {
                SetMode::Add
            } else if mode.starts_with("-") {
                SetMode::Remove
            } else {
                error_resp!(Command::ERR_UMODEUNKNOWNFLAG(
                    Responses::UModeUnknownFlag { nick: user.nick().clone() },
                ));
            };

            let m = mode.chars()
                .nth(1)
                .unwrap()
                .to_string()
                .parse::<UserMode>()
                .map_err(|_| {
                    error_resp!(Command::ERR_UMODEUNKNOWNFLAG(
                        Responses::UModeUnknownFlag { nick: user.nick().clone() },
                    ))
                })
                .unwrap();
            user.set_mode(&set, &vec![m]);
            Vec::new()
        }

        Command::PING(Requests::Ping { originator, target }) => {
            if target.is_some() && target.unwrap() != configuration.hostname {
                unimplemented!();
            }
            vec![
                Message {
                    prefix: None,
                    command: Command::PONG(Requests::Pong {
                        originator: configuration.hostname.clone(),
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

pub fn process_broadcast(
    configuration: Arc<Configuration>,
    connection: Arc<Mutex<Connection>>,
    b: Arc<Broadcast>,
) -> Vec<Message> {
    trace!("Connection state: {:?}.", connection);
    match b.deref() {
        &Broadcast::Join(ref user, ref channel) => {
            vec![
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
            ]
        }
        &Broadcast::Part(_, _, _) => unimplemented!(),
        /*result.push(Message {
                prefix: None,
                command: Command::PART(Requests::Part {
                    channels: vec![c.clone()],
                    message: None,
                }),
            });*/
        &Broadcast::PrivateMessage => unimplemented!(),
        u @ _ => {
            error!("Broadcast message {:?} not yet implemented.", u);
            Vec::new()
        }
    }
}


// Returns WELCOME sequence.
fn welcome_sequence(configuration: &Configuration, user: &User) -> Vec<Message> {
    #[derive(Serialize)]
    struct WelcomeData<'a> {
        network_name: &'a str,
        nick: &'a str,
    }

    vec![
        Message {
            prefix: None,
            command: Command::RPL_WELCOME(Responses::Welcome {
                nick: user.nick().clone(),
                message: configuration
                    .template_engine
                    .render(
                        templates::RPL_WELCOME_TEMPLATE_NAME,
                        &WelcomeData {
                            network_name: &configuration.network_name,
                            nick: user.nick(),
                        },
                    )
                    .unwrap(),
            }),
        },
        Message {
            prefix: None,
            command: Command::RPL_YOURHOST(Responses::YourHost {
                nick: user.nick().clone(),
                message: configuration
                    .template_engine
                    .render(templates::RPL_YOURHOST_TEMPLATE_NAME, &configuration)
                    .unwrap(),
            }),
        },
        Message {
            prefix: None,
            command: Command::RPL_CREATED(Responses::Created {
                nick: user.nick().clone(),
                message: configuration
                    .template_engine
                    .render(templates::RPL_CREATED_TEMPLATE_NAME, &configuration)
                    .unwrap(),
            }),
        },
        Message {
            prefix: None,
            command: Command::RPL_MYINFO(Responses::MyInfo::default()),
        },
    ]
}
