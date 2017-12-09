use chrono;
use handlebars;
use hostname;

use serde::ser::{self, SerializeSeq};

use std::fmt;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::{templates, Broadcast};
use super::user::Identifier as UserIdentifier;

use futures::sync::mpsc;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    NickInUse,
    UnknownUser,
    Other,
}

#[derive(Serialize)]
pub struct Server {
    #[serde(serialize_with = "chrono_datetime_serializer")]
    pub created: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub hostname: String,
    pub network_name: String,

    #[serde(skip)]
    pub template_engine: handlebars::Handlebars,

    #[serde(skip)]
    // All known users are stored here.
    users: RwLock<HashMap<UserIdentifier, mpsc::Sender<Arc<Broadcast>>>>,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "created: {:?} version: {:?} hostname: {:?}, network_name: {:?} users: {:?}",
            self.created,
            self.version,
            self.hostname,
            self.network_name,
            self.users
        )
    }
}

pub fn chrono_datetime_serializer<S, X>(
    t: &chrono::DateTime<X>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    X: chrono::TimeZone,
{
    serializer.serialize_str(&format!("{:?}", t))
}

/*#[derive(Debug, Serialize)]
pub struct Channel {
    pub name: String,
    pub topic: Option<String>,
    #[serde(serialize_with = "member_serializer")]
    pub users: HashMap<UserIdentifier, RefCell<mpsc::Sender<Arc<Broadcast>>>>,
    key: Option<String>,
}

pub fn member_serializer<S>(
    t: &HashMap<UserIdentifier, RefCell<mpsc::Sender<Arc<Broadcast>>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    let mut s = serializer.serialize_seq(Some(t.len()))?;
    for u in t.keys() {
        s.serialize_element(&u.nickname)?;
    }
    s.end()
}

impl Channel {
    pub fn new(name: &String) -> Self {
        Channel {
            name: name.clone(),
            topic: None,
            users: HashMap::new(),
            key: None,
        }
    }

    // Should only be used for comparison.
    pub fn from_string(name: &String) -> Self {
        Self::new(name)
    }

    pub fn verify_key(&self, key: Option<&String>) -> bool {
        self.key.as_ref() == key
    }
}

impl std::cmp::PartialEq for Channel {
    fn eq(&self, other: &Channel) -> bool {
        self.name == other.name
    }
}

impl std::cmp::Eq for Channel {}

impl std::hash::Hash for Channel {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.name.hash(state)
    }
}*/

/*macro_rules! send_to_user {
    ($user:expr, $tx:expr, $message:expr) => {
        if let Err(ref e) = $tx.borrow_mut().try_send($message) {
            if e.is_disconnected() {
                error!("Trying to broadcast to dropped TX for {:?}.", $user);
            } else if e.is_full() {
                error!("Trying to broadcast to full TX for {:?}.", $user);
            }
        }
    };
}*/

impl Server {
    pub fn new(time: chrono::DateTime<chrono::Utc>, version: String) -> Self {
        let mut template_engine = handlebars::Handlebars::new();
        macro_rules! register_template {
            ($name:ident, $template:ident) => {
                template_engine.register_template_string(templates::$name, templates::$template).unwrap();
            }
        }
        // Register all known templates.
        register_template!(DEBUG_TEMPLATE_NAME, DEBUG_HTML_TEMPLATE);
        register_template!(RPL_WELCOME_TEMPLATE_NAME, RPL_WELCOME_TEMPLATE);
        register_template!(RPL_YOURHOST_TEMPLATE_NAME, RPL_YOURHOST_TEMPLATE);
        register_template!(RPL_CREATED_TEMPLATE_NAME, RPL_CREATED_TEMPLATE);

        /* For rapid template iteration (only bin restart required).
        template_engine.register_template_file(
                templates::DEBUG_TEMPLATE_NAME,
                "./template",
            ).unwrap();
       */

        Self {
            created: time,
            version: version,
            hostname: hostname::get_hostname().expect("unable to get hostname"),
            network_name: "IRC Network".to_string(),
            template_engine: template_engine,
            users: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_user(
        &self,
        user: &UserIdentifier,
        tx: mpsc::Sender<Arc<Broadcast>>,
    ) -> Result<(), ServerError> {
        let users = self.users.write().unwrap();
        if users.contains_key(user) {
            return Err(ServerError::NickInUse);
        }
        users.insert(user.clone(), tx);
        Ok(())
    }

    pub fn remove_user(&self, user: &UserIdentifier) -> Result<(), ServerError> {
        match self.users.write().unwrap().remove(user) {
            Some(_) => Ok(()),
            None => Err(ServerError::UnknownUser),
        }
    }

    // Replaces old_nick with new_nick for user.
    pub fn replace_nick(
        &self,
        old: &UserIdentifier,
        new: &UserIdentifier,
    ) -> Result<(), ServerError> {
        debug!(
            "Replacing nick [{:?}] with [{:?}].",
            old,
            new,
        );
        let users = self.users.write().unwrap();
        if users.contains_key(new) {
            return Err(ServerError::NickInUse);
        }
        users.insert(new.clone(), users.remove(old).unwrap());
        Ok(())
    }

    /*pub fn join(&mut self, user: &UserIdentifier, channels: Vec<(&String, Option<&String>)>) {
        let tx = self.users.get(user).unwrap().clone();
        for &(channel_name, key) in channels.iter() {
            if !self.channels.contains_key(channel_name) {
                self.channels.insert(
                    channel_name.clone(),
                    Channel::new(channel_name),
                );
            }
            let mut channel = self.channels.get_mut(channel_name).unwrap();

            if !channel.verify_key(key) {
                error!("Cannot join {:?}: wrong key.", channel);
                continue;
            }

            if channel.users.insert(user.clone(), tx.clone()).is_none() {
                // TODO(permission checks and all that).
                let msg = Arc::new(Broadcast::Join(user.clone(), channel.name.clone()));
                let mut dropped = Vec::new();
                for (user, tx) in channel.users.iter() {
                    if let Err(ref e) = tx.borrow_mut().try_send(Arc::clone(&msg)) {
                        if e.is_disconnected() {
                            dropped.push(user.clone());
                        }
                    }
                }
                for d in dropped {
                    channel.users.remove(&d);
                }
            } else {
                trace!("User {:?} already in channel {}.", user, channel_name);
            }
        }
    }

    pub fn part(
        &mut self,
        user: &UserIdentifier,
        channels: &Vec<String>,
        message: &Option<String>,
    ) {
        let tx = self.users.get(user).unwrap().clone();
        for c in channels.iter() {
            let chan = self.channels.get_mut(c);
            if chan.is_none() {
                warn!("Trying to part non-existant channel {}", c);
                continue;
            }
            let mut chan = chan.unwrap();
            if !chan.users.remove(user).is_some() {
                trace!(
                    "{:?} cannot part channel {} they're not a member of.",
                    user,
                    c
                );
                continue;
            }

            let msg = Arc::new(Broadcast::Part(
                user.clone(),
                chan.name.clone(),
                message.clone(),
            ));
            let mut dropped = Vec::new();
            for (user, tx) in chan.users.iter() {
                if let Err(ref e) = tx.borrow_mut().try_send(Arc::clone(&msg)) {
                    if e.is_disconnected() {
                        dropped.push(user.clone());
                    }
                }
            }
            for d in dropped {
                chan.users.remove(&d);
            }
            send_to_user!(user, tx, Arc::clone(&msg));
        }
    }

    pub fn part_all(&mut self, user: &UserIdentifier, message: Option<&String>) {
        //unimplemented!()
    }*/
}
