use futures::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;
use std::{self, fmt, str};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use super::messages::Message as IRCMessage;
use super::messages::commands::{Command, requests as Requests, responses as Responses};
use super::connection::ConnectionTX;
use super::channel::{Identifier as ChannelIdentifier, ChannelError, Channel};
use super::server::Server;

#[derive(Debug)]
pub enum Message {
    // (User, Channel).
    Join(String),
    // (User, Channel, Message)
    Part(String, Option<String>),
    PrivateMessage,

    // Channel responses.
    /*ChannelJoin(
        Result<
            (ChannelIdentifier, Option<String>, Vec<UserIdentifier>, ChannelTX),
            (ChannelError, ChannelIdentifier),
        >
    ),*/

    // User responses.
    //UserModeChanged(Arc<(user::Identifier, SetMode, Vec<UserMode>)>),

    // Server Responses.
    //ServerRegistrationResult(Option<ServerError>),
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Identifier {
    nickname: String,
    username: String,
    realname: String,
    hostname: String,
}

impl Identifier {
    pub fn new(nickname: String, username: String, realname: String, hostname: String) -> Self {
        Self {
            nickname,
            username,
            realname,
            hostname,
        }
    }
    pub fn nick(&self) -> &String {
        &self.nickname
    }
    pub fn into_nick(self) -> String {
        self.nickname
    }
}

#[derive(Debug, Serialize)]
pub struct User {
    ident: Identifier,
    modes: HashSet<UserMode>,
    #[serde(skip)]
    channels: HashSet<ChannelIdentifier>,
    #[serde(skip)]
    server: Arc<Mutex<Server>>,
    #[serde(skip)]
    tx: ConnectionTX,
}

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

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.nickname)
    }
}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Identifier) -> bool {
        assert!(self.nickname.len() > 0);
        self.nickname == other.nickname
    }
}

impl std::cmp::Eq for Identifier {}

impl std::hash::Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nickname.hash(state);
    }
}

impl Identifier {
    pub fn as_prefix(&self) -> String {
        format!("{}!{}@{}", self.nickname, self.username, self.hostname)
    }
}

impl std::cmp::PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.ident == other.ident
    }
}

impl std::cmp::Eq for User {}

macro_rules! send_log_err {
    ($tx:expr, $message:expr) => {
        match $tx.try_send($message) {
            Err(e) => debug!("Send error: {:?}.", e),
            _ =>{},
        };
    }
}

impl User {
    pub fn new(ident: &Identifier, server: Arc<Mutex<Server>>, tx: ConnectionTX) -> Self {
        Self {
            ident: ident.clone(),
            modes: HashSet::new(),
            channels: HashSet::new(),
            server: server,
            tx: tx,
        }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.ident
    }

    pub fn nick(&self) -> &String {
        &self.ident.nickname
    }

    pub fn join(&mut self, channel: &ChannelIdentifier) {
        assert!(self.channels.insert(channel.clone()));
    }

    pub fn part(&mut self, channel: &ChannelIdentifier) {
        assert!(self.channels.remove(&channel));
    }

    pub fn channels<'a>(&'a self) -> std::collections::hash_set::Iter<'a, ChannelIdentifier> {
        self.channels.iter()
    }

    pub fn set_mode(&mut self, set: &SetMode, mode: &Vec<UserMode>) -> Vec<IRCMessage> {
        let mut modified = Vec::new();
        for m in mode.iter() {
            if match set {
                &SetMode::Add => self.modes.insert(m.clone()),
                &SetMode::Remove => self.modes.remove(&m),
            }
            {
                modified.push(m.clone());
            }
        }
        trace!(
            "{} mode is {}.",
            self.nick(),
            self.modes
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );

        if modified.len() == 0 {
            return Vec::new();
        }

        let mut mode_string = String::new();
        for m in modified {
            mode_string = format!("{}{}", mode_string, m);
        }
        match set {
            &SetMode::Add => mode_string = format!("+{}", mode_string),
            &SetMode::Remove => mode_string = format!("-{}", mode_string),
        };
        vec![
            IRCMessage {
                prefix: None,
                command: Command::MODE(Requests::Mode {
                    target: self.nick().clone(),
                    mode_string: Some(mode_string),
                    mode_args: None,
                }),
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub enum SetMode {
    Add,
    Remove,
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

impl fmt::Display for UserMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                &UserMode::Away => "a",
                &UserMode::Invisible => "i",
                &UserMode::WallOps => "w",
                &UserMode::Restricted => "r",
                &UserMode::Operator => "o",
                &UserMode::LocalOperator => "O",
                &UserMode::ServerNotices => "s",
            }
        )
    }
}
