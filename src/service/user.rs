use std::{self, fmt, str};

// Stored in the server.
#[derive(Debug, Serialize, Default, Clone, Hash)]
pub struct Identifier {
    nickname: String,
    username: String,
    realname: String,
}

#[derive(Debug, Serialize)]
pub struct User {
    ident: Identifier,

    pub modes: HashSet<UserMode>,
    pub channels: HashSet<String>,
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
        write!(f, "{}", self.nick)?;
        if let (Some(ref u), Some(ref h)) = (self.username, self.hostname)  {
            write!(f, "!{}@{}", u, h)?;
        }
        Ok(())
    }
}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Identifier) -> bool {
        self.nickname.len() > 0 && self.nickname == other.nickname
    }
}

impl Identifier {
    pub fn nick(&self) -> &String {
        self.nickname.as_ref().unwrap()
    }
}

impl std::cmp::PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.ident == other.iden
    }
}

impl User {
    pub fn new(nickname: String, username: String, realname: String) -> Self {
        User {
            ident: Identifier { nickname, username, realname },
            modes: HashSet::new(),
            channels: HashSet::new(),
        }
    }

    pub fn identifier(&self) -> Identifier {
        self.ident.clone()
    }

    // Returns Vec<(Channel name, Topic, Vec<Nicks>)>.
    pub fn join(&mut self, channels: Vec<(&String, Option<&String>)>) {
        let mut server = self.server.lock().unwrap();
        for &(c, key) in channels.iter() {
            if self.channels.contains(c) {
                warn!(
                    "{} trying to join the same channel {}.",
                    self.nick.as_ref().unwrap(),
                    c
                );
            }
            server.join(&self, c, key);
        }
    }

    pub fn part(&mut self, channels: &Vec<String>, message: &Option<String>) {
        let mut server = self.server.lock().unwrap();
        for c in channels {
            if !self.channels.remove(c) {
                warn!("Failed to part {:?}.", c);
                continue;
            }
            server.part(c, &message);
        }
    }

    pub fn part_all(&mut self, message: &Option<String>) {
        self.part(self.channels.iter().cloned().collect(), message);
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
