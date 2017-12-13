use futures::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;
use std::{self, fmt, str};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use super::server::ServerTX;
use super::connection::{Message as ConnectionMessage, ConnectionTX};
use super::channel::{Identifier as ChannelIdentifier, ChannelError, Channel};
use super::shared_state::SharedState;

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

pub type UserTX = mpsc::Sender<Message>;

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
    shared_state: Arc<SharedState>,
    modes: HashSet<UserMode>,
    channels: HashSet<ChannelIdentifier>,
    connection_tx: ConnectionTX,
    server_tx: ServerTX,
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

pub fn new(
    ident: &Identifier,
    shared_state: Arc<SharedState>,
    connection_tx: ConnectionTX,
    server_tx: ServerTX,
    thread_pool: CpuPool,
) -> UserTX {
    let (tx, rx) = mpsc::channel(shared_state.configuration.user_message_queue_length);
    let user = Arc::new(Mutex::new(User {
        ident: ident.clone(),
        shared_state: shared_state,
        modes: HashSet::new(),
        channels: HashSet::new(),
        connection_tx: connection_tx,
        server_tx: server_tx,
    }));

    thread_pool
        .spawn(rx.for_each(move |message| {
            let mut user = user.lock().unwrap();
            debug!("User {} processing {:?}.", user.identifier(), message);
            match message {
                /*Message::Join(user, tx, key) => {
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
                            let msg =
                                ConnectionMessage::ChannelJoin(Err((err, channel.ident.clone())));
                            send_log_err!(tx, msg);
                        }
                    }
                }*/
                u @ _ => error!("{:?} not yet implemented!", u),
            }
            Ok(())
        }))
        .forget();
    tx
}

impl User {
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
            /*let message = ConnectionMessage::UserModeChanged(
                Arc::new((self.identifier().clone(), set.clone(), modified)),
            );*/
            // TODO(lazau): Should we broadcast to all Channels too?
            //send_log_err!(self.connection_tx, message);
        }
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
