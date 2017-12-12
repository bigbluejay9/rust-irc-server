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

static CHANNEL_MPSC_LENGTH: usize = 20;

#[derive(Debug)]
pub enum Message {
    Join(
        UserIdentifier,
        ConnectionTX,
        /*Key*/
        Option<String>
    ),
}

pub type ChannelTX = mpsc::Sender<Message>;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug)]
struct Channel {
    ident: Identifier,
    topic: Option<String>,
    users: HashMap<UserIdentifier, ConnectionTX>,
    banned: HashSet<UserIdentifier>,
    key: Option<String>,
    tx: ChannelTX,
}

impl Identifier {
    pub fn from_name(name: &String) -> Self {
        Self { name: name.clone() }
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

pub fn new(ident: Identifier, shared_state: Arc<SharedState>, thread_pool: &CpuPool) -> ChannelTX {
    let (tx, rx) = mpsc::channel(shared_state.configuration.channel_message_queue_length);
    let channel = Arc::new(Mutex::new(Channel {
        ident: ident,
        topic: None,
        users: HashMap::new(),
        banned: HashSet::new(),
        key: None,
        tx: tx.clone(),
    }));

    thread_pool
        .spawn(
            rx.and_then(move |message| {
                let mut channel = channel.lock().unwrap();
                debug!("Channel {} processing {:?}.", channel.name(), message);
                match message {
                    Message::Join(user, tx, key) => {
                        match channel.try_join(&user, tx, key) {
                            Ok(tx) => {
                                let msg = ConnectionMessage::ChannelJoin(Ok((
                                    channel.ident.clone(),
                                    channel.topic.clone(),
                                    channel.users.keys().cloned().collect(),
                                    channel.tx.clone(),
                                )));
                                send_log_err!(channel.lookup_user_mut(&user).unwrap(), msg);
                            }
                            Err((err, mut tx)) => {
                                let msg = ConnectionMessage::ChannelJoin(
                                    Err((err, channel.ident.clone())),
                                );
                                send_log_err!(tx, msg);
                            }
                        }
                    }
                    u @ _ => error!("{:?} not yet implemented!", u),
                }
                Ok(())
            }).collect(),
        )
        .forget();
    tx
}

#[derive(Debug)]
pub enum ChannelError {
    BadKey,
    Banned,
    AlreadyMember,
}

impl Channel {
    fn name(&self) -> &String {
        &self.ident.name
    }

    fn identifier(&self) -> &Identifier {
        &self.ident
    }

    fn verify_key(&self, key: Option<String>) -> bool {
        self.key.as_ref() == key.as_ref()
    }

    fn lookup_user_mut(&mut self, user: &UserIdentifier) -> Option<&mut ConnectionTX> {
        self.users.get_mut(user)
    }

    fn try_join(
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
