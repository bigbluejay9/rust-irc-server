use futures::sync::mpsc;

use serde::ser::{self, SerializeSeq};

use std;
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;

use super::Broadcast;
use super::user::Identifier as UserIdentifier;

#[derive(Debug, Serialize)]
pub struct Channel {
    pub name: String,
    pub topic: Option<String>,
    #[serde(serialize_with = "member_serializer")]
    pub users: HashMap<UserIdentifier, RefCell<mpsc::Sender<Arc<Broadcast>>>>,
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
}

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
