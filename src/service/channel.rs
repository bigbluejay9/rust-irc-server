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
    users: HashSet<UserIdentifier>,
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
    pub fn new(ident: Identifier, shared_state: Arc<SharedState>) -> Self {
        Self {
            ident: ident,
            topic: None,
            users: HashSet::new(),
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

    pub fn topic(&self) -> &Option<String> {
        &self.topic
    }

    pub fn verify_key(&self, key: Option<&String>) -> bool {
        self.key.as_ref() == key
    }

    /*pub fn lookup_user_mut(&mut self, user: &UserIdentifier) -> Option<&mut ConnectionTX> {
        self.users.get_mut(user)
    }*/

    pub fn users<'a>(&'a self) -> std::collections::hash_set::Iter<'a, UserIdentifier> {
        self.users.iter()
    }

    pub fn join(
        &mut self,
        user: &UserIdentifier,
        key: Option<&String>,
    ) -> Result<(), ChannelError> {
        if !self.verify_key(key) {
            return Err(ChannelError::BadKey);
        }

        if self.banned.contains(user) {
            return Err(ChannelError::Banned);
        }

        if self.users.contains(user) {
            return Err(ChannelError::AlreadyMember);
        }

        //self.broadcast
        self.users.insert(user.clone());
        Ok(())
    }

    //pub fn broadcast(&self,
}
