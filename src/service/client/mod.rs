mod processor;

pub use self::processor::process_message;

use std::ops::DerefMut;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use super::server::{Server, Channel};
use super::SocketPair;
use super::messages::Message;
use super::messages::commands::Command;
use super::messages::commands::requests as Requests;
use super::messages::commands::responses as Responses;

#[derive(Debug, Serialize)]
pub struct User {
    pub username: String,
    pub realname: String,
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

    // Returns Vec<(Channel name, Topic, Vec<Nicks>)>.
    pub fn join(&mut self, channels: Vec<(&String, Option<&String>)>) -> Vec<Message> {
        let mut server = self.server.lock().unwrap();
        let mut result = Vec::new();
        let mut chans: Vec<&Channel> = Vec::new();
        for &(c, key) in channels.iter() {
            let chan = server.join(&self, c, key);
            result.push(Message {
                prefix: None,
                command: Command::JOIN(Requests::Join {
                    join: Requests::JoinChannels::Channels(vec![c.clone()]),
                }),
            });
            result.push(Message {
                prefix: None,
                command: Command::RPL_TOPIC(Responses::Topic {
                    nick: self.nick.unwrap(),
                    channel: c.clone(),
                    topic: chan.topic.clone(),
                }),
            });
            result.push(Message {
                prefix: None,
                command: Command::RPL_NAMREPLY(Responses::NamReply {
                    nick: self.nick.unwrap(),
                    // XXX
                    symbol: "".to_string(),
                    channel: c.clone(),
                    // xxx
                    members: chan.nicks
                        .iter()
                        .cloned()
                        .map(|n| ("".to_string, n))
                        .collect(),
                }),
            });
            match chan {
                Ok(chan) => chans.push(chan),
                Err(e) => warn!("Failed to join channel {}: {:?}.", c, e),
            };
        }
        /*chans
            .into_iter()
            .flat_map(|chan| {
                let mut resp = vec![
                    Message {
                        prefix: "".to_string(),
                        command: Command::JOIN(Requests::Join {
                            join: Requests::JoinChannels::Channels(vec![chan.clone()]),
                        }),
                    },
                    Message {
                        prefix: "".to_string(),
                        command: Command::RPL_TOPIC(Responses::Topic {
                            nick: self.nick.unwrap(),
                            channel: chan.clone(),
                            topic: topic,
                        }),
                    },
                ];

                for n in nicks.into_iter() {
                    resp.push(Message {
                        prefix: "".to_string(),
                        command: Command::RPL_NAMREPLY(Responses::NamReply {
                            nick: self.nick.unwrap(),
                            symbol: "".to_string(),
                            channel: chan,
                            // (Prefix, Nick).
                            members: Vec::new(),
                        }),
                    });
                }
            })
            .collect();*/
        unimplemented!()
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
