use chrono;
use hostname;

use serde::ser;

use std::collections::{HashSet, HashMap};
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

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
pub enum ServerError {
    NickInUse,
    Other,
}

#[derive(Debug, Serialize)]
pub struct Server {
    // Really this is the static part of the server, a.k.a. static config.
    #[serde(serialize_with = "chrono_datetime_serializer")]
    pub created: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub hostname: String,
    #[serde(skip)]
    nicks: HashSet<String>,

    channels: HashMap<String, String>,

    // The heart of the server (note that nothing is serialized...).
    #[serde(skip)]
    pub nick_to_client: HashMap<String, SocketPair>,
    #[serde(skip)]
    clients: HashMap<SocketPair, mpsc::Sender<String>>,

    // Only used for stats generation.
    // Lock order:
    // 1. Client.
    // 2. Server.
    // Do not lock the clients in this map while holding Server lock, rather copy the arcs into a
    // different data structure and lock them once you've let go of the Server lock.
    #[serde(skip)]
    pub connections: HashMap<SocketPair, Arc<Mutex<Client>>>,
}

#[derive(Debug, Serialize)]
pub struct Client {
    // Unique per Client.
    pub socket: SocketPair,

    pub nick: Option<String>,
    pub user: Option<User>,

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

impl Server {
    pub fn new(time: chrono::DateTime<chrono::Utc>, version: String) -> Self {
        Server {
            created: time,
            version: version,
            hostname: hostname::get_hostname().expect("unable to get hostname"),
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

    pub fn join_channel(
        &mut self,
        client: &Client,
        channel: &String,
        key: Option<&String>,
    ) -> (String, Vec<String>) {
        if client.nick.is_none() {}
        //if self.channels.contains(client.nick
        unimplemented!()
    }
}

impl Client {
    pub fn new(addr: SocketPair, server: Arc<Mutex<Server>>) -> Self {
        Client {
            socket: addr,
            nick: None,
            user: None,
            server: server,
        }
    }

    pub fn registered(&self) -> bool {
        self.nick.is_some() && self.user.is_some()
    }
}
