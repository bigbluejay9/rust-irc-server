use futures::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;
use std;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use super::channel::{Channel, Identifier as ChannelIdentifier};
use super::connection::{Message as ConnectionMessage, ConnectionTX};
use super::user::Identifier as UserIdentifier;
use super::shared_state::SharedState;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    NickInUse,
    UnknownUser,
    Other,
}

#[derive(Debug)]
pub struct Server {
    // Unlike Channel/Connection,
    // All known users.
    users: HashMap<UserIdentifier, ConnectionTX>,
    // All known channels.
    channels: HashMap<ChannelIdentifier, Channel>,
    shared_state: Arc<SharedState>,
}

impl Server {
    pub fn new(shared_state: Arc<SharedState>) -> Self {
        // TODO(lazau): Load preconfigured channels.
        Self {
            users: HashMap::new(),
            channels: HashMap::new(),
            shared_state: shared_state,
        }
    }

    pub fn add_user(&mut self, user: &UserIdentifier, tx: ConnectionTX) -> Result<(), ServerError> {
        debug!("Inserting {:?} into {:?}.", user, self.users);
        if self.users.contains_key(user) {
            return Err(ServerError::NickInUse);
        }
        self.users.insert(user.clone(), tx);
        Ok(())
    }

    pub fn remove_user(&mut self, user: &UserIdentifier) {
        if self.users.remove(user).is_none() {
            warn!("Removing unknown user: {:?}.", user);
        }
    }

    // Replaces old_nick with new_nick for user.
    pub fn replace_nick(
        &mut self,
        old: &UserIdentifier,
        new: &UserIdentifier,
    ) -> Result<(), ServerError> {
        debug!(
            "Replacing nick [{:?}] with [{:?}].",
            old,
            new,
        );
        if self.users.contains_key(new) {
            return Err(ServerError::NickInUse);
        }
        let removed = self.users.remove(old).unwrap();
        self.users.insert(new.clone(), removed);
        Ok(())
    }

    pub fn lookup_channel(&mut self, channel: &ChannelIdentifier) -> &Channel {
        debug!("Looking up channel: {:?}.", channel);
        if !self.channels.contains_key(&channel) {
            self.channels.insert(
                channel.clone(),
                Channel::new(channel, Arc::clone(&self.shared_state)),
            );
        }
        self.channels.get(&channel).unwrap()
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
