use chrono;
use handlebars;
use hostname;

use serde::ser;

use std;
use std::ops::DerefMut;
use std::collections::{HashSet, HashMap};
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::templates;

use futures::sync::mpsc;

// Used to identify clients.
// Server is represented by (local, local) pair.
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct SocketPair {
    pub local: SocketAddr,
    pub remote: SocketAddr,
}

impl fmt::Display for SocketPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({} : {})", self.local, self.remote)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NickInUse,
    TryingToPartNonmemberChannel,
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

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Channel {
    name: String,
    topic: String,
    nicks: HashSet<String>,
}

impl std::hash::Hash for Channel {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.name.hash(state)
    }
}

// Mutable data for the server.
#[derive(Debug)]
pub struct Server {
    nicks: HashSet<String>,

    // Channel name -> Channel.
    channels: HashMap<String, Channel>,

    pub nick_to_client: HashMap<String, SocketPair>,
    clients: HashMap<SocketPair, mpsc::Sender<String>>,

    // Only used for stats generation.
    // Lock order:
    // 1. Client.
    // 2. Server.
    // Do not lock the clients in this map while holding Server lock, rather copy the arcs into a
    // different data structure and lock them once you've let go of the Server lock.
    pub connections: HashMap<SocketPair, Arc<Mutex<Client>>>,
}

#[derive(Debug, Serialize)]
pub struct Client {
    // Unique per Client.
    pub socket: SocketPair,

    pub nick: Option<String>,
    pub user: Option<User>,

    pub channels: HashSet<String>,

    // Implicity enforce locking order by only allowing Server access through client (thereby
    // ensuring that the Client lock is held before the server).
    #[serde(skip)]
    pub server: Arc<Mutex<Server>>,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub username: String,
    pub realname: String,
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
            topic: "".to_string(),
            nicks: HashSet::new(),
        }
    }
}

impl Server {
    pub fn new() -> Self {
        Server {
            nicks: HashSet::new(),
            channels: HashMap::new(),
            nick_to_client: HashMap::new(),
            clients: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    // Replaces old_nick with new_nick for client.
    pub fn replace_nick(
        &mut self,
        client: &Client,
        new_nick: String,
    ) -> Result<Option<String>, Error> {
        debug!(
            "Replacing nick [{:?}] with [{:?}] for {:?}.",
            client.nick,
            new_nick,
            client
        );
        if self.nicks.contains(&new_nick) {
            return Err(Error::NickInUse);
        }

        let old_nick_clone = client.nick.clone();
        if let Some(ref old) = client.nick {
            self.nicks.remove(old);
            assert_eq!(self.nick_to_client.remove(old).unwrap(), client.socket);
        }

        self.nicks.insert(new_nick.clone());
        self.nick_to_client.insert(
            new_nick.clone(),
            client.socket.clone(),
        );
        Ok(old_nick_clone)
    }

    pub fn insert_client(
        &mut self,
        socket: &SocketPair,
        client: Arc<Mutex<Client>>,
        tx: mpsc::Sender<String>,
    ) {
        assert!(self.clients.insert(socket.clone(), tx).is_none());
        assert!(self.connections.insert(socket.clone(), client).is_none());
    }

    pub fn remove_client(&mut self, client: &Client) {
        if let Some(ref nick) = client.nick {
            assert!(self.nicks.remove(nick));
            assert_eq!(&self.nick_to_client.remove(nick).unwrap(), &client.socket);
        }
        assert!(self.clients.remove(&client.socket).is_some());
        assert!(self.connections.remove(&client.socket).is_some());
    }

    fn lookup_channel(&self, channel: &String) -> Option<&Channel> {
        self.channels.get(channel)
    }

    pub fn join(
        &mut self,
        client: &Client,
        channel: &String,
        key: Option<&String>,
    ) -> Result<&Channel, Error> {
        if self.channels.contains_key(channel) {
            unimplemented!()
        } else {
            let new_channel = Channel::new(channel);
            assert!(self.channels.insert(channel.clone(), new_channel).is_none());
            Ok(self.channels.get(channel).unwrap())
        }
    }

    pub fn remove_client_from_channel(&mut self, channel: &String, client: &Client) {
        assert!(
            self.channels
                .get_mut(channel)
                .expect("trying to remove client from unknown channel")
                .nicks
                .remove(client.nick.as_ref().expect(
                    "trying to remove a client from channel without a nickname",
                ))
        );
    }
}

impl Client {
    pub fn new(addr: SocketPair, server: Arc<Mutex<Server>>) -> Self {
        Client {
            socket: addr,
            nick: None,
            user: None,
            server: server,
            channels: HashSet::new(),
        }
    }

    pub fn registered(&self) -> bool {
        self.nick.is_some() && self.user.is_some()
    }

    pub fn join(
        &mut self,
        channels: Vec<(&String, Option<&String>)>,
    ) -> Vec<(String, Vec<String>)> {
        let mut server = self.server.lock().unwrap();
        let mut result = Vec::new();
        for &(c, key) in channels.iter() {
            let chan = server.join(&self, c, key);
            match chan {
                Ok(chan) => result.push((chan.topic.clone(), chan.nicks.iter().cloned().collect())),
                Err(e) => warn!("Failed to join channel {}: {:?}.", c, e),
            };
        }
        result
    }

    pub fn part(&mut self, channel: &String, server: &mut Server) {
        if !self.channels.remove(channel) {
            warn!("Trying to part non-existant channel {}.", channel);
            return;
        }
        server.remove_client_from_channel(channel, self);
    }

    pub fn part_all(&mut self) {
        let cloned = Arc::clone(&self.server);
        let mut server = cloned.lock().unwrap();
        let channels_cloned = self.channels.clone();
        for chan in channels_cloned.iter() {
            self.part(chan, server.deref_mut());
        }
    }
}
