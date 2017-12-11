use futures::*;
use futures::sync::mpsc;
use futures_cpupool::CpuPool;
use std;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use super::channel::{Message as ChannelMessage, Identifier as ChannelIdentifier, ChannelTX};
use super::connection::{Message as ConnectionMessage, ConnectionTX};
use super::user::Identifier as UserIdentifier;
use super::shared_state::SharedState;

#[derive(Debug)]
pub enum Message {
    Register(UserIdentifier, ConnectionTX),
    Join(UserIdentifier, ChannelIdentifier),
}

pub type ServerTX = mpsc::Sender<Message>;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    NickInUse,
    UnknownUser,
    Other,
}

#[derive(Debug)]
struct Server {
    // Unlike Channel/Connection,
    // All known users.
    users: Mutex<HashMap<UserIdentifier, ConnectionTX>>,
    // All known channels.
    channels: Mutex<HashMap<ChannelIdentifier, ChannelTX>>,

    tx: ServerTX,
    shared_state: Arc<SharedState>,
    thread_pool: CpuPool,
}

pub fn new(shared_state: Arc<SharedState>, thread_pool: CpuPool) -> ServerTX {
    let (tx, rx) = mpsc::channel(shared_state.configuration.server_message_queue_length);
    let server = Server {
        users: Mutex::new(HashMap::new()),
        channels: Mutex::new(HashMap::new()),
        shared_state: shared_state,
        tx: tx.clone(),
        thread_pool: thread_pool.clone(),
    };

    thread_pool
        .spawn_fn(move || {
            rx.then(move |message| -> future::FutureResult<(), ()> {
                let message = message.unwrap();
                debug!("Processing server message {:?}.", message);
                match message {
                    Message::Register(user, tx) => {
                        match server.add_user(&user, tx) {
                            Ok(_) => {
                                server
                                    .users
                                    .lock()
                                    .unwrap()
                                    .get_mut(&user)
                                    .unwrap()
                                    .try_send(
                                        Arc::new(ConnectionMessage::ServerRegistrationResult(None)),
                                    )
                                    .map_err(|e| debug!("Send error: {:?}.", e));
                            }
                            Err(e) => {
                                server
                                    .users
                                    .lock()
                                    .unwrap()
                                    .get_mut(&user)
                                    .unwrap()
                                    .try_send(Arc::new(
                                        ConnectionMessage::ServerRegistrationResult(Some(e)),
                                    ))
                                    .map_err(|e| debug!("Send error: {:?}.", e));
                            }
                        }
                    }
                    Message::Join(user, channel) => unimplemented!(),
                    _ => unimplemented!(),
                }
                future::ok(())
            }).collect()
        })
        .forget();
    tx
}

impl Server {
    pub fn add_user(&self, user: &UserIdentifier, tx: ConnectionTX) -> Result<(), ServerError> {
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

impl std::ops::Drop for Server {
    fn drop(&mut self) {
        debug!("Server is closing.\nFinal state: {:#?}.", self);
    }
}
