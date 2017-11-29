use chrono;

use serde::ser;

use std::collections::{HashSet, HashMap};
use std::net::SocketAddr;

use futures::sync::mpsc;

#[derive(Debug, Serialize)]
pub struct Server {
    #[serde(serialize_with = "chrono_datetime_serializer")]
    pub created: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub hostname: String,
    pub nicknames: HashSet<String>,

    // LOCK ORDER. LOCK CLIENT FIRST, THEN SERVER. KAPEESH?
    // OTHERWISE YOU'LL DEADLOCK.
    #[serde(skip)]
    pub connections: HashMap<(SocketAddr, SocketAddr), mpsc::Sender<String>>,
}

#[derive(Debug, Serialize)]
pub struct Connection {
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,

    pub nick: Option<String>,
    pub user: Option<User>,
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
