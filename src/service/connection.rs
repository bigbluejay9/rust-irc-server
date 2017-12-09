pub use self::processor::{process_message, process_broadcast};

use std::iter;
use std::{fmt, str};
use std::clone::Clone;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use super::{SocketPair, User, Broadcast};
use super::server::{Configuration, Server, ServerError};
use super::messages::Message;
use super::messages::commands::Command;
use super::messages::commands::requests as Requests;
use super::messages::commands::responses as Responses;

enum ConnectionType {
    // Unkown type. After registration is complete will be either Client or Server.
    Registering(Registration),

    Client(User),
    // TODO(lazau): To support server connections, use something like HashSet<User>.
    Server, // unimplemented.
}

#[derive(Debug, Serialize, Default)]
struct Registration {
    nickname: Option<String>,
    username: Option<String>,
    realname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Connection {
    // Unique per Connection.
    pub socket: SocketPair,

    conn_type: ConnectionType,

    // Implicity enforce locking order by only allowing Server access through Connection.
    // (thereby ensuring that the Connection lock is held before the server).
    #[serde(skip)]
    pub server: Arc<Mutex<Server>>,
}

impl Connection {
    pub fn new(addr: SocketPair, server: Arc<Mutex<Server>>) -> Self {
        Connection {
            socket: addr,
            conn_type: ConnectionType::Registering(Registration::default()),
            server: server,
        }
    }

    pub fn try_register(&mut self) -> bool {
        //assert_eq!(
        // make sure we're in registering state.
        // TODO XXX
        //
    }

    pub fn registered(&self) -> bool {
        match self.conn_type {
            ConnectionType::Registering(_) => false,
            _ => true,
        }
    }

    pub fn disconnect(&mut self) -> Self {
        if let Some(c) = self.conn_type {
            let ident = match c {
                ConnectionType::Client(ref u) => {
                    debug!("Removing user, disconnecing: {:?}.", u);
                    u.identifier()
                }
                ConnectionType::Server => unimplemented!()
            }
            self.server.lock().unwrap().replace_user(&ident, None);
        }
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
            if let Some(t) = $connection.conn_type {
                match t {
                    ConnectionType::Client(ref u) => u,
                    ConnectionType::Server => unimplemented!(),
                }
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
            if conn
                .server
                .lock()
                .unwrap()
                .replace_nick(&connection, nick.clone())
                .err() == Some(ServerError::NickInUse)
            {
                error_resp!(Command::ERR_NICKNAMEINUSE(
                    Responses::NICKNAMEINUSE { nick: nick.clone() },
                ));

            }
            connection.user.nick = Some(nick);

            maybe_welcome_sequence(&configuration, &connection)
        }

        Command::USER(Requests::User {
                          username,
                          mode: _mode,
                          unused: _unused,
                          realname,
                      }) => {
            let mut connection = connection.lock().unwrap();
            if connection.user.registered() {
                error_resp!(Command::ERR_ALREADYREGISTRED(
                    Responses::AlreadyRegistered {
                        nick: connection.user.nick.unwrap().clone(),
                    },
                ));
            }
            connection.user.user_real_name = Some((username, realname));

            maybe_welcome_sequence(&configuration, &connection)
        }

        Command::JOIN(Requests::Join { join: jt }) => {
            let mut connection = connection.lock().unwrap();
            verify_registered!(connection);
            match jt {
                Requests::JoinChannels::PartAll => {
                    connection.user.part_all();
                }
                Requests::JoinChannels::Channels(r) => {
                    connection.user.join(
                        r.iter().zip(iter::repeat(None)).collect(),
                    );
                }
                Requests::JoinChannels::KeyedChannels(r) => {
                    let (chans, keys): (Vec<String>, Vec<String>) = r.into_iter().unzip();
                    connection.user.join(
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
            verify_registered!(connection);
            connection.user.part(&channels, &message);
            Vec::new()
        }

        Command::MODE(Requests::Mode {
                          target,
                          mode_string,
                          mode_args,
                      }) => {
            let mut connection = connection.lock().unwrap();
            verify_registered!(connection);
            if &target != connection.user.nick.as_ref().unwrap() {
                error_resp!(Command::ERR_USERSDONTMATCH(Responses::UsersDontMatch {
                    nick: connection.user.nick.clone().unwrap(),
                }));
            }
            if mode_string.is_none() {
                unimplemented!();
            }
            let mode = mode_string.unwrap();
            if mode.len() < 2 {
                error_resp!(Command::ERR_UMODEUNKNOWNFLAG(Responses::UModeUnknownFlag {
                    nick: connection.user.nick.clone().unwrap(),
                }));
            }
            let set = if mode.starts_with("+") {
                super::SetMode::Add
            } else if mode.starts_with("-") {
                super::SetMode::Remove
            } else {
                error_resp!(Command::ERR_UMODEUNKNOWNFLAG(Responses::UModeUnknownFlag {
                    nick: connection.user.nick.clone().unwrap(),
                }));
            };

            let m = mode.chars()
                .nth(1)
                .unwrap()
                .to_string()
                .parse::<super::UserMode>()
                .map_err(|_| {
                    error_resp!(Command::ERR_UMODEUNKNOWNFLAG(Responses::UModeUnknownFlag {
                        nick: connection.user.nick.clone().unwrap(),
                    }))
                })
                .unwrap();
            connection.user.set_mode(&set, &vec![m]);
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
        &Broadcast::Part => unimplemented!(),
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
    if !connection.user.registered() {
        return Vec::new();
    }

    #[derive(Serialize)]
    struct WelcomeData<'a> {
        network_name: &'a str,
        nick: &'a str,
    }

    vec![
        Message {
            prefix: None,
            command: Command::RPL_WELCOME(Responses::Welcome {
                nick: connection.user.nick.as_ref().unwrap().clone(),
                message: configuration
                    .template_engine
                    .render(
                        templates::RPL_WELCOME_TEMPLATE_NAME,
                        &WelcomeData {
                            network_name: &configuration.network_name,
                            nick: connection.user.nick.as_ref().unwrap(),
                        },
                    )
                    .unwrap(),
            }),
        },
        Message {
            prefix: None,
            command: Command::RPL_YOURHOST(Responses::YourHost {
                nick: connection.user.nick.as_ref().unwrap().clone(),
                message: configuration
                    .template_engine
                    .render(templates::RPL_YOURHOST_TEMPLATE_NAME, &configuration)
                    .unwrap(),
            }),
        },
        Message {
            prefix: None,
            command: Command::RPL_CREATED(Responses::Created {
                nick: connection.user.nick.as_ref().unwrap().clone(),
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
