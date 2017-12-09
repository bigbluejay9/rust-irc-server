use chrono;
use handlebars;
use hostname;

use serde::ser;

use std;
use std::cell::{RefMut, RefCell};
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;

use super::{templates, SocketPair, Broadcast};
use super::connection::Connection;
use super::user::Identifier as UserIdentifier;

use futures::sync::mpsc;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    NickInUse,
    UnknownUser,
    Other,
}

// Immutable data for the server.
#[derive(Serialize)]
pub struct Configuration {
    // Really this is the static part of the server, a.k.a. static config.
    #[serde(serialize_with = "chrono_datetime_serializer")]
    pub created: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub hostname: String,
    pub network_name: String,

    #[serde(skip)]
    pub template_engine: handlebars::Handlebars,
}

#[derive(Debug, Serialize, Eq)]
pub struct Channel {
    pub name: String,
    pub topic: Option<String>,
    pub users: HashSet<UserIdentifier>,
}

// Mutable data for the server.
#[derive(Debug)]
pub struct Server {
    // Channel name -> Channel.
    pub channels: HashMap<Channel>,

    // All known users are stored here.
    // User -> TX.
    pub users: HashMap<UserIdentifier, RefCell<mpsc::Sender<Arc<Broadcast>>>>
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

impl Configuration {
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

        Configuration {
            created: time,
            version: version,
            hostname: hostname::get_hostname().expect("unable to get hostname"),
            network_name: "IRC Network".to_string(),
            template_engine: template_engine,
        }
    }
}

impl Channel {
    pub fn new(name: &String) -> Self {
        Channel {
            name: name.clone(),
            topic: None,
            nicks: HashSet::new(),
        }
    }
    
    // Should only be used for comparison.
    pub fn from_string(name: &String) -> Self {
        new(name)
    }
}

impl std::cmp::PartialEq for Channel {
    fn eq(&self, other: &Channel) -> bool {
        self.name == other.name
    }
}

impl std::hash::Hash for Channel {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.name.hash(state)
    }
}

impl Server {
    pub fn new() -> Self {
        Server {
            channels: HashMap::new(),
            users: HashMap::new(),
        }
    }

    // Replaces old_nick with new_nick for user.
    pub fn replace_user(
        &mut self,
        old: &UserIdentifier,
        new: &Option<UserIdentifier>,
    ) -> Result<(), ServerError> {
        debug!(
            "Replacing nick [{:?}] with [{:?}] for {:?}.",
            old,
            new,
            user
        );
        if let Some(ref n) = new && self.users.contains(n) {
            return Err(ServerError::NickInUse);
        }
        let tx = self.users.remove(&old).unwrap();

        if let Some(ref n) = new {
            self.users.insert(new, tx);
        } else {
            debug!("Dropping user: {:?}.", old);
        }
        Ok(())
    }

    fn lookup_channel(&self, channel: &String) -> Option<&Channel> {
        self.channels.get(channel)
    }

    fn user_to_tx(&self, user: &UserIdentifier) -> Option<RefMut<mpsc::Sender<Arc<Broadcast>>>> {
        if let Some(s) = self.users.get(user) {
            return Some(s.borrow_mut());
        }
        None
    }

    fn send_to_user(&self, user: &UserIdentifier, message: Arc<Broadcast>) {
        if let Some(ref mut s) = self.user_to_tx(user) {
            if let Err(ref e) = s.try_send(message) {
                if e.is_disconnected() {
                    error!("Trying to broadcast to dropped TX for {}.", nick);
                } else if e.is_full() {
                    error!("Trying to broadcast to full TX for {}.", nick);
                }
            }
        } else {
            warn!("Can't broadcast {:?} to {}.", message, nick);
        }
    }

    pub fn join(&mut self, user: &UserIdentifier, channel: &String, key: Option<&String>) {
        let channel = Channel::from_string(channel);
        if !self.channels.contains(&channel) {
            self.channels.insert(channel.clone(), Channel::new(channel));
        }
        // TODO(permission checks and all that).
        let chan = self.channels.get(&channel).unwrap();
        let msg = Arc::new(Broadcast::Join((
        user.nick().clone(), chan.name.clone()
        )));

        for user in chan.users.iter() {
            self.send_to_user(user, Arc::clone(&msg));
        }
    }

    pub fn part(&mut self, channel: &String, message: &Option<String>) {
        //unimplemented!()
    }
}
