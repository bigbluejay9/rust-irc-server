use std;
use std::collections::HashMap;
use std::sync::Arc;
use super::channel::{Channel, Identifier as ChannelIdentifier, ChannelError};
use super::connection::ConnectionTX;
use super::user::Identifier as UserIdentifier;
use super::shared_state::SharedState;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    NickInUse,
    UnknownUser,
    NoSuchChannel,
    NotOnChannel,
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

    pub fn lookup_channel(&mut self, channel: &ChannelIdentifier) -> Option<&Channel> {
        debug!("Looking up channel: {:?}.", channel);
        self.channels.get(&channel)
    }

    pub fn channels<'a>(
        &'a self,
    ) -> std::collections::hash_map::Iter<'a, ChannelIdentifier, Channel> {
        self.channels.iter()
    }

    fn lookup_user(&self, user: &UserIdentifier) -> Option<&ConnectionTX> {
        self.users.get(user)
    }

    pub fn users<'a>(
        &'a self,
    ) -> std::collections::hash_map::Keys<'a, UserIdentifier, ConnectionTX> {
        self.users.keys()
    }

    pub fn join(
        &mut self,
        user: &UserIdentifier,
        channels: &Vec<(String, Option<String>)>,
    ) -> Vec<Result<(Option<String>, Vec<UserIdentifier>), ChannelError>> {
        let mut result = Vec::with_capacity(channels.len());
        let tx = self.lookup_user(user).unwrap().clone();
        for &(ref channel_name, ref key) in channels.iter() {
            let ident = ChannelIdentifier::from_name(channel_name);
            if !self.channels.contains_key(&ident) {
                self.channels.insert(
                    ident.clone(),
                    Channel::new(ident.clone(), Arc::clone(&self.shared_state)),
                );
            }
            let channel = self.channels.get_mut(&ident).unwrap();
            match channel.join(user, &tx, key) {
                Ok(_) => {
                    result.push(Ok((
                        channel.topic().clone(),
                        channel.users().cloned().collect(),
                    )))
                }
                Err(e) => result.push(Err(e)),
            };
        }
        result
    }

    pub fn part(
        &mut self,
        user: &UserIdentifier,
        channels: &Vec<String>,
        message: &Option<String>,
    ) -> Vec<Result<(), ServerError>> {
        let mut result = Vec::with_capacity(channels.len());
        for c in channels {
            let ident = ChannelIdentifier::from_name(c);
            if !self.channels.contains_key(&ident) {
                result.push(Err(ServerError::NoSuchChannel));
                continue;
            }

            let channel = self.channels.get_mut(&ident).unwrap();
            if !channel.has_user(user) {
                result.push(Err(ServerError::NotOnChannel));
                continue;
            }

            channel.part(user, message);
            result.push(Ok(()));
        }
        result
    }

    pub fn send(&mut self, user: &UserIdentifier, targets: &Vec<String>, message: &String) {
        for t in targets {
            self.channels
                .get_mut(&ChannelIdentifier::from_name(t))
                .unwrap()
                .privmsg(user, message);
        }
    }
}
