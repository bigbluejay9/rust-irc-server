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
use super::server::{Server, ServerError};
use super::templates;
use super::user::{SetMode, User, UserMode, Identifier as UserIdentifier};

#[derive(Debug)]
enum ConnectionType {
    // Unkown type. After registration is complete will be either Client or Server.
    Registering(Registration),
    Client(User),
    Server, // unimplemented.
}

impl std::default::Default for ConnectionType {
    fn default() -> Self {
        ConnectionType::Registering(Registration::default())
    }
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
    server: Arc<Server>,
    tx: Option<mpsc::Sender<Arc<Broadcast>>>,
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
                Message {
prefix: None,
            command: $err,
                },
            ];
        };
    }

impl Connection {
    pub fn new(addr: SocketPair, server: Arc<Server>, tx: mpsc::Sender<Arc<Broadcast>>) -> Self {
        Connection {
            socket: addr,
            conn_type: ConnectionType::default(),
            server: server,
            tx: Some(tx),
        }
    }

    fn try_register(&mut self) -> Vec<Message> {
        assert!(self.registered().is_none());
        let ident;
        if let ConnectionType::Registering(r) = self.conn_type {
            if r.nickname.is_none() || r.username.is_none() || r.realname.is_none() {
                return Vec::new();
            } else {
                ident = &UserIdentifier {
                    nickname: r.nickname.unwrap(),
                    username: r.username.unwrap(),
                    realname: r.realname.unwrap(),
                };
            }
        }

        let tx = self.tx.take().unwrap();
        match self.server.add_user(ident, tx) {
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
                    Message {
                        prefix: None,
                        command: Command::RPL_WELCOME(Responses::Welcome {
                            nick: nick.clone(),
                            message: self.server
                                .template_engine
                                .render(
                                    templates::RPL_WELCOME_TEMPLATE_NAME,
                                    &WelcomeData {
                                        network_name: &self.server.network_name,
                                        nick: &nick,
                                    },
                                )
                                .unwrap(),
                        }),
                    },
                    Message {
                        prefix: None,
                        command: Command::RPL_YOURHOST(Responses::YourHost {
                            nick: nick.clone(),
                            message: self.server
                                .template_engine
                                .render(templates::RPL_YOURHOST_TEMPLATE_NAME, self.server.deref())
                                .unwrap(),
                        }),
                    },
                    Message {
                        prefix: None,
                        command: Command::RPL_CREATED(Responses::Created {
                            nick: nick.clone(),
                            message: self.server
                                .template_engine
                                .render(templates::RPL_CREATED_TEMPLATE_NAME, self.server.deref())
                                .unwrap(),
                        }),
                    },
                    Message {
                        prefix: None,
                        command: Command::RPL_MYINFO(Responses::MyInfo::default()),
                    },
                ];
            }
            Err(ServerError::NickInUse) => {
                self.tx = Some(tx);
                return_error_resp!(Command::ERR_NICKNAMEINUSE(
                    Responses::NICKNAMEINUSE { nick: ident.nickname.clone() },
                ));
            }
            _ => unreachable!(),
        }
    }

    fn add_registration_info(
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

    fn registered(&self) -> Option<&User> {
        match self.conn_type {
            ConnectionType::Registering(_) => None,
            ConnectionType::Client(ref u) => Some(u),
            ConnectionType::Server => unimplemented!(),
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(ref u) = self.registered() {
            self.server.remove_user(u.identifier()).unwrap();
        }
        //TODO(lazau): Maybe gotta part all channels?
    }

    pub fn process_message(&mut self, req: Message) -> Vec<Message> {
        trace!("Connection state: {:?}.", self);

        macro_rules! verify_registered_and_return_user {
            () => {
                if let Some(ref mut u) = self.registered() {
                    u
                } else {
                    return_error_resp!(Command::ERR_NOTREGISTERED(Responses::NOTREGISTERED::default()));
                }
            }
        }

        match req.command {
            Command::NICK(Requests::Nick { nickname: nick }) => {
                // TODO(lazau): Validate nick based on
                // https://tools.ietf.org/html/rfc2812#section-2.3.1.
                if let Some(user) = self.registered() {
                    let mut new_ident = user.identifier().clone();
                    new_ident.nickname = nick.clone();
                    match self.server.replace_nick(user.identifier(), &new_ident) {
                        Err(ServerError::NickInUse) => {
                            return_error_resp!(Command::ERR_NICKNAMEINUSE(
                                Responses::NICKNAMEINUSE { nick: nick.clone() },
                            ));
                        }
                        // NICK change.
                        Ok(_) => unimplemented!(),
                        _ => unreachable!(),
                    }
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
                if let Some(ref u) = self.registered() {
                    return_error_resp!(Command::ERR_ALREADYREGISTRED(
                        Responses::AlreadyRegistered { nick: u.nick().clone() },
                    ));
                } else {
                    self.add_registration_info(None, Some(username), Some(realname));
                    self.try_register()
                }
            }

            Command::JOIN(Requests::Join { join: jt }) => {
                let mut user = verify_registered_and_return_user!();
                match jt {
                    Requests::JoinChannels::PartAll => {
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
                if target.is_some() && target.unwrap() != self.server.hostname {
                    // Forward to another server.
                    unimplemented!();
                }
                vec![
                    Message {
                        prefix: None,
                        command: Command::PONG(Requests::Pong {
                            originator: self.server.hostname.clone(),
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

    pub fn process_broadcast(&mut self, b: Arc<Broadcast>) -> Vec<Message> {
        trace!("Connection state: {:?}.", self);
        match b.deref() {
            &Broadcast::Join(ref user, ref channel) => {
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
}
