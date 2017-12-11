use futures::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;

use serde::ser::{self, SerializeSeq};

use std;
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use super::user::Identifier as UserIdentifier;
use super::connection::ConnectionTX;

static CHANNEL_MPSC_LENGTH: usize = 20;

#[derive(Debug, Serialize)]
pub enum Message {
    Join, // unimplemented
}

pub type ChannelTX = mpsc::Sender<Arc<Message>>;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug)]
struct Channel {
    ident: Identifier,
    pub topic: Option<String>,
    pub users: HashMap<UserIdentifier, RefCell<ConnectionTX>>,
    key: Option<String>,
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

impl Channel {
    pub fn new(name: &String, thread_pool: &CpuPool) -> mpsc::Sender<Arc<Message>> {
        let (tx, rx) = mpsc::channel(CHANNEL_MPSC_LENGTH);
        let chan = Arc::new(Mutex::new(Channel {
            ident: Identifier { name: name.clone() },
            topic: None,
            users: HashMap::new(),
            key: None,
        }));

        thread_pool
            .spawn(
                rx.and_then(move |message| {
                    let chan = chan.lock().unwrap();
                    debug!("Channel {} processing {:?}.", chan.name(), message);
                    Ok(())
                }).collect(),
            )
            .forget();
        tx
    }

    fn name(&self) -> &String {
        &self.ident.name
    }

    fn verify_key(&self, key: Option<&String>) -> bool {
        self.key.as_ref() == key
    }
}
