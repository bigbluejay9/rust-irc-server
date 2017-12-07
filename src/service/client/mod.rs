mod processor;

pub use self::processor::{process_message, process_broadcast};

use std::{fmt, str};
use std::clone::Clone;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use super::server::Server;
use super::SocketPair;
use super::messages::Message;
use super::messages::commands::Command;
use super::messages::commands::requests as Requests;
use super::messages::commands::responses as Responses;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
pub enum UserMode {
    Away,
    Invisible,
    WallOps,
    Restricted,
    Operator,
    LocalOperator,
    ServerNotices,
}

impl str::FromStr for UserMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 1 {
            return Err(());
        }

        match s.as_ref() {
            "a" => Ok(UserMode::Away),
            "i" => Ok(UserMode::Invisible),
            "w" => Ok(UserMode::WallOps),
            "r" => Ok(UserMode::Restricted),
            "o" => Ok(UserMode::Operator),
            "O" => Ok(UserMode::LocalOperator),
            "s" => Ok(UserMode::ServerNotices),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserPrefix {
    pub nick: String,
    pub user_host: Option<(String, String)>,
}

impl fmt::Display for UserPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.nick)?;
        if let Some((ref u, ref h)) = self.user_host {
            write!(f, "!{}@{}", u, h)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct User {
    pub username: String,
    pub realname: String,
}

#[derive(Debug, Serialize)]
pub struct Client {
    // Unique per Client.
    pub socket: SocketPair,

    pub nick: Option<String>,
    pub user: Option<User>,

    pub modes: HashSet<UserMode>,

    pub channels: HashSet<String>,

    // Implicity enforce locking order by only allowing Server access through client (thereby
    // ensuring that the Client lock is held before the server).
    #[serde(skip)]
    pub server: Arc<Mutex<Server>>,
}

pub enum SetMode {
    Add,
    Remove,
}

impl Client {
    pub fn new(addr: SocketPair, server: Arc<Mutex<Server>>) -> Self {
        Client {
            socket: addr,
            nick: None,
            user: None,
            modes: HashSet::new(),
            server: server,
            channels: HashSet::new(),
        }
    }

    pub fn registered(&self) -> bool {
        self.nick.is_some() && self.user.is_some()
    }

    pub fn user_prefix(&self) -> UserPrefix {
        UserPrefix {
            nick: self.nick.as_ref().unwrap().clone(),
            user_host: Some((
                self.user.as_ref().unwrap().username.clone(),
                "XXX".to_string(),
            )),
        }
    }

    // Returns Vec<(Channel name, Topic, Vec<Nicks>)>.
    pub fn join(&mut self, channels: Vec<(&String, Option<&String>)>) -> Vec<Message> {
        let mut server = self.server.lock().unwrap();
        let mut result = Vec::new();
        for &(c, key) in channels.iter() {
            if self.channels.contains(c) {
                warn!(
                    "{} trying to join the same channel {}.",
                    self.nick.as_ref().unwrap(),
                    c
                );
            }
            let chan = server.join(&self, c, key);
            if chan.is_err() {
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
            });
        }
        result
    }

    pub fn part(&mut self, channels: &Vec<String>, message: &Option<String>) -> Vec<Message> {
        let mut result = Vec::new();
        let mut server = self.server.lock().unwrap();
        for c in channels {
            if !self.channels.remove(c) {
                warn!("Failed to part {:?}.", c);
                continue;
            }
            server.part(c, &message);
            result.push(Message {
                prefix: None,
                command: Command::PART(Requests::Part {
                    channels: vec![c.clone()],
                    message: None,
                }),
            });
        }
        result
    }

    pub fn part_all(&mut self) {
        error!("JOIN 0 (part all) not implemented!");
        /*let cloned = Arc::clone(&self.server);
        let mut server = cloned.lock().unwrap();
        let channels_cloned = self.channels.clone();
        for chan in channels_cloned.iter() {
            self.part(chan, server.deref_mut());
        }*/
    }

    pub fn set_mode(&mut self, set: &SetMode, mode: &Vec<UserMode>) {
        for m in mode.iter() {
            match set {
                &SetMode::Add => self.modes.insert(m.clone()),
                &SetMode::Remove => self.modes.remove(m),
            };
        }
    }
}
