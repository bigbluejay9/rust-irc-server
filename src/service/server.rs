use futures_cpupool::CpuPool;

use std::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use super::Broadcast;
use super::channel::{Message as ChannelMessage, Identifier as ChannelIdentifier};
use super::user::Identifier as UserIdentifier;
use super::shared_state::SharedState;

use futures::sync::mpsc;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
}

pub type ServerTX = mpsc::Sender::Arc<Message>;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    NickInUse,
    UnknownUser,
    Other,
}

#[derive(Debug)]
struct Server {
    // All known users.
    users: Mutex<HashMap<UserIdentifier, mpsc::Sender<Arc<Broadcast>>>>,

    // All known channels.
    channels: Mutex<HashMap<ChannelIdentifier, mpsc::Sender<Arc<ChannelMessage>>>>,

    thread_pool: CpuPool,
}

pub fn new(shared_state: Arc<SharedState>, thread_pool: CpuPool) -> ServerTX {}

impl Server {
    pub fn new(thread_pool: &CpuPool) -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
            thread_pool: thread_pool.clone(),
        }
    }

    pub fn add_user(
        &self,
        user: &UserIdentifier,
        tx: mpsc::Sender<Arc<Broadcast>>,
    ) -> Result<(), ServerError> {
        let mut users = self.users.lock().unwrap();
        if users.contains_key(user) {
            return Err(ServerError::NickInUse);
        }
        users.insert(user.clone(), tx);
        Ok(())
    }

    pub fn remove_user(&self, user: &UserIdentifier) -> Result<(), ServerError> {
        match self.users.lock().unwrap().remove(user) {
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
        let mut users = self.users.lock().unwrap();
        if users.contains_key(new) {
            return Err(ServerError::NickInUse);
        }
        let removed = users.remove(old).unwrap();
        users.insert(new.clone(), removed);
        Ok(())
    }

    pub fn lookup_channel(
        &self,
        channel: &ChannelIdentifier,
    ) -> Result<mpsc::Sender<Arc<ChannelMessage>>, ServerError> {
        debug!("Looking up channel: {:?}.", channel);
        let mut channels = self.channels.lock().unwrap();
        if channels.contains_key(channel) {
            return Ok(channels.get(channel).unwrap().clone());
        }

        // Create new channel.
        self.create_channel(channels, channel)
    }

    fn create_channel(
        &self,
        channels: MutexGuard<HashMap<ChannelIdentifier, mpsc::Sender<Arc<ChannelMessage>>>>,
        channel: &ChannelIdentifier,
    ) -> Result<mpsc::Sender<Arc<ChannelMessage>>, ServerError> {
        unimplemented!()
        //assert!(channels.insert(channel.clone
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
