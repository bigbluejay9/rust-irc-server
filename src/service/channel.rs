use futures::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;

use serde::ser::{self, SerializeSeq};

use std;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use super::user::Identifier as UserIdentifier;
use super::connection::{ConnectionTX, Message as ConnectionMessage};
use super::shared_state::SharedState;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Identifier {
    name: String,
}

#[derive(Debug)]
pub struct Channel {
    ident: Identifier,
    topic: Option<String>,
    users: HashMap<UserIdentifier, ConnectionTX>,
    banned: HashSet<UserIdentifier>,
    key: Option<String>,
}

impl Identifier {
    pub fn from_name(name: &String) -> Self {
        Self { name: name.clone() }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl std::cmp::PartialEq for Channel {
    fn eq(&self, other: &Channel) -> bool {
        self.ident == other.ident
    }
}
impl std::cmp::Eq for Channel {}

impl std::hash::Hash for Channel {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.ident.hash(state)
    }
}

macro_rules! send_log_err {
    ($tx:expr, $message:expr) => {
        match $tx.try_send($message) {
            Err(e) => debug!("Send error: {:?}.", e),
            _ =>{},
        };
    }
}

#[derive(Debug)]
pub enum ChannelError {
    BadKey,
    Banned,
    AlreadyMember,
}

impl Channel {
    pub fn new(ident: &Identifier, shared_state: Arc<SharedState>) -> Self {
        Self {
            ident: ident.clone(),
            topic: None,
            users: HashMap::new(),
            banned: HashSet::new(),
            key: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.ident.name
    }

    pub fn identifier(&self) -> &Identifier {
        &self.ident
    }

    pub fn verify_key(&self, key: Option<String>) -> bool {
        self.key.as_ref() == key.as_ref()
    }

    pub fn lookup_user_mut(&mut self, user: &UserIdentifier) -> Option<&mut ConnectionTX> {
        self.users.get_mut(user)
    }

    pub fn users<'a>(
        &'a self,
    ) -> std::collections::hash_map::Keys<'a, UserIdentifier, ConnectionTX> {
        self.users.keys()
    }

    pub fn try_join(
        &mut self,
        user: &UserIdentifier,
        tx: ConnectionTX,
        key: Option<String>,
    ) -> Result<(), (ChannelError, ConnectionTX)> {
        if !self.verify_key(key) {
            return Err((ChannelError::BadKey, tx));
        }
        if self.banned.contains(user) {
            return Err((ChannelError::Banned, tx));
        }
        if self.users.contains_key(user) {
            return Err((ChannelError::AlreadyMember, tx));
        }
        self.users.insert(user.clone(), tx);
        Ok(())
    }
}
