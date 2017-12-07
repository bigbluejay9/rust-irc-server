use chrono;
use handlebars;
use hostname;

use serde::ser;

use std;
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};

use super::{templates, SocketPair, Broadcast};
use super::client::Client;

use futures::sync::mpsc;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    NickInUse,
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
    pub name: String,
    pub topic: String,
    pub nicks: HashSet<String>,
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
    clients: HashMap<SocketPair, mpsc::Sender<Arc<Broadcast>>>,

    // Only used for stats generation.
    // Lock order:
    // 1. Client.
    // 2. Server.
    // Do not lock the clients in this map while holding Server lock, rather copy the arcs into a
    // different data structure and lock them once you've let go of the Server lock.
    pub connections: HashMap<SocketPair, Arc<Mutex<Client>>>,
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
    ) -> Result<Option<String>, ServerError> {
        debug!(
            "Replacing nick [{:?}] with [{:?}] for {:?}.",
            client.nick,
            new_nick,
            client
        );
        if self.nicks.contains(&new_nick) {
            return Err(ServerError::NickInUse);
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
        tx: mpsc::Sender<Arc<Broadcast>>,
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

    fn lookup_nick(&mut self, nick: &String) -> Option<&mut mpsc::Sender<Arc<Broadcast>>> {
        if let Some(ref sp) = self.nick_to_client.get(nick) {
            if let Some(s) = self.clients.get_mut(sp) {
                return Some(s);
            }
        }
        None
    }

    fn try_send_broadcast(&mut self, nick: &String, message: Arc<Broadcast>) {
        if let Some(ref mut s) = self.lookup_nick(nick) {
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

    pub fn join(
        &mut self,
        client: &Client,
        channel: &String,
        key: Option<&String>,
    ) -> Result<&Channel, ServerError> {
        if self.channels.contains_key(channel) {
            let chan = self.channels.get(channel).unwrap();
            let msg = Arc::new(Broadcast::Join(
                client.user_prefix(),
                client.nick.as_ref().unwrap().clone(),
            ));
            for n in chan.nicks.iter() {
                //self.try_send_broadcast(n, Arc::clone(&msg));
                unimplemented!()
            }
        // TODO(permission checks and all that).
        } else {
            let new_channel = Channel::new(channel);
            assert!(self.channels.insert(channel.clone(), new_channel).is_none());
        }
        Ok(self.channels.get(channel).unwrap())
    }

    pub fn part(&mut self, channel: &String, message: &Option<String>) {
        //unimplemented!()
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
