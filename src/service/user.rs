use std::{self, fmt, str};
use std::collections::{HashMap, HashSet};

use super::channel::Channel;

// Stored in the server.
#[derive(Debug, Serialize, Default, Clone, Hash, Eq)]
pub struct Identifier {
    pub nickname: String,
    pub username: String,
    pub realname: String,
}

#[derive(Debug, Serialize)]
pub struct User {
    ident: Identifier,

    pub modes: HashSet<UserMode>,
    pub channels: HashMap<String, Channel>,
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

impl std::cmp::PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.ident == other.ident
    }
}

impl User {
    pub fn new(nickname: String, username: String, realname: String) -> Self {
        User {
            ident: Identifier {
                nickname,
                username,
                realname,
            },
            modes: HashSet::new(),
            channels: HashMap::new(),
        }
    }

    pub fn identifier(&self) -> &Identifier {
        &self.ident
    }

    pub fn nick(&self) -> &String {
        &self.ident.nickname
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
