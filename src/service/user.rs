use std::{self, fmt, str};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use super::connection::{Message as ConnectionMessage, ConnectionTX};
use super::channel::{Identifier as ChannelIdentifier, Message as ChannelMessage, ChannelTX};

// Stored in the server.
#[derive(Debug, Default, Clone)]
pub struct Identifier {
    pub nickname: String,
    pub username: String,
    pub realname: String,
    pub hostname: String,
}

#[derive(Debug)]
pub struct User {
    ident: Identifier,
    connection_tx: ConnectionTX,
    modes: HashSet<UserMode>,
    channels: HashMap<ChannelIdentifier, ChannelTX>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
    pub fn new(
        nickname: &String,
        username: &String,
        realname: &String,
        hostname: &String,
        connection_tx: ConnectionTX,
    ) -> Self {
        User {
            ident: Identifier {
                nickname: nickname.clone(),
                username: username.clone(),
                realname: realname.clone(),
                hostname: hostname.clone(),
            },
            modes: HashSet::new(),
            channels: HashMap::new(),
            connection_tx: connection_tx,
        }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.ident
    }

    pub fn nick(&self) -> &String {
        &self.ident.nickname
    }

    pub fn set_mode(&mut self, set: &SetMode, mode: &Vec<UserMode>) {
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
        debug!(
            "{} mode is {}.",
            self.nick(),
            self.modes
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(",")
        );

        if modified.len() > 0 {
            let message = ConnectionMessage::UserModeChanged(
                Arc::new((self.identifier().clone(), set.clone(), modified)),
            );
            // TODO(lazau): Should we broadcast to all Channels too?
            send_log_err!(self.connection_tx, message);
        }
    }

    pub fn joined_channel(&mut self, channel: &ChannelIdentifier, tx: ChannelTX) {
        assert!(self.channels.insert(channel.clone(), tx).is_none());
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
