use futures::*;
use serde::ser::{self, SerializeSeq};
use std;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use super::connection::{ConnectionTX, Event};
use super::messages::Message as IRCMessage;
use super::messages::commands::{Command, requests as Requests};
use super::shared_state::SharedState;
use super::user::Identifier as UserIdentifier;

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
    shared_state: Arc<SharedState>,
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
            users: HashMap::new(),
            banned: HashSet::new(),
            key: None,
            shared_state: shared_state,
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

    pub fn verify_key(&self, key: &Option<String>) -> bool {
        &self.key == key
    }

    pub fn users<'a>(
        &'a self,
    ) -> std::collections::hash_map::Keys<'a, UserIdentifier, ConnectionTX> {
        self.users.keys()
    }

    pub fn has_user(&self, user: &UserIdentifier) -> bool {
        self.users.contains_key(user)
    }

    pub fn join(
        &mut self,
        user: &UserIdentifier,
        tx: &ConnectionTX,
        key: &Option<String>,
    ) -> Result<(), ChannelError> {
        if !self.verify_key(key) {
            return Err(ChannelError::BadKey);
        }

        if self.banned.contains(user) {
            return Err(ChannelError::Banned);
        }

        if self.users.contains_key(user) {
            return Err(ChannelError::AlreadyMember);
        }

        self.broadcast(
            None,
            Event::Message(vec![
                IRCMessage {
                    prefix: Some(user.as_prefix()),
                    command: Command::JOIN(Requests::Join {
                        join: Requests::JoinChannels::Channels(vec![self.name().clone()]),
                    }),
                },
            ]),
        );
        self.users.insert(user.clone(), tx.clone());
        Ok(())
    }

    pub fn part(&mut self, user: &UserIdentifier, message: &Option<String>) {
        assert!(self.users.remove(user).is_some());
        self.broadcast(
            None,
            Event::Message(vec![
                IRCMessage {
                    prefix: Some(user.as_prefix()),
                    command: Command::PART(Requests::Part {
                        channels: vec![self.name().clone()],
                        message: message.clone(),
                    }),
                },
            ]),
        );
    }

    pub fn privmsg(&mut self, source: &UserIdentifier, message: &String) {
        self.broadcast(
            Some(source.clone()),
            Event::Message(vec![
                IRCMessage {
                    prefix: Some(source.as_prefix()),
                    command: Command::PRIVMSG(Requests::Privmsg {
                        targets: vec![self.name().clone()],
                        message: message.clone(),
                    }),
                },
            ]),
        );
    }

    fn broadcast(&self, skip_user: Option<UserIdentifier>, message: Event) {
        self.users
            .iter()
            .filter(|&(u, _)| if let Some(ref skip) = skip_user {
                u != skip
            } else {
                true
            })
            .for_each(|(_, tx)| {
                let tx = tx.clone();
                let message = message.clone();
                self.shared_state
                    .thread_pool
                    .spawn_fn(move || tx.send(message))
                    .forget()
            });
    }

    //pub fn broadcast(&self,
}
