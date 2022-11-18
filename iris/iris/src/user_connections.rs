use anyhow::anyhow;
use common::connect::ConnectionWrite;
use common::types::{Channel, ErrorType, Nick, Target};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

pub struct UserConnections {
    channels_per_user: BTreeMap<Nick, BTreeSet<Channel>>,
    users_per_channel: BTreeMap<Channel, BTreeSet<Nick>>,
    writers: BTreeMap<Nick, Arc<Mutex<ConnectionWrite>>>,
}

impl UserConnections {
    pub fn new() -> UserConnections {
        UserConnections {
            users_per_channel: BTreeMap::new(),
            channels_per_user: BTreeMap::new(),
            writers: BTreeMap::new(),
        }
    }

    pub fn add_user(
        &mut self,
        nick: &Nick,
        conn_write: Arc<Mutex<ConnectionWrite>>,
    ) -> anyhow::Result<()> {
        if let Some(_) = self.writers.insert(nick.clone(), conn_write) {
            Err(ErrorType::NickCollision)
        } else {
            Ok(())
        }
        .map_err(|e| anyhow!(e))
    }

    pub fn remove_user(&mut self, nick: &Nick) {
        self.writers.remove(nick);
        if let Some(channels) = self.channels_per_user.get(&nick.clone()) {
            for channel in channels.iter() {
                self.users_per_channel
                    .get_mut(channel)
                    .map(|nicks| nicks.remove(nick));
            }
        }

        self.channels_per_user.remove(nick);
    }

    pub fn add_user_to_channel(&mut self, nick: &Nick, channel: &Channel) -> anyhow::Result<()> {
        if !self.writers.contains_key(nick) {
            panic!("User {nick} does not already exist before being added to channel {channel}");
        }

        self.users_per_channel
            .entry(channel.clone())
            .or_insert(BTreeSet::new())
            .insert(nick.clone());
        self.channels_per_user
            .entry(nick.clone())
            .or_insert(BTreeSet::new())
            .insert(channel.clone());

        Ok(())
    }

    pub fn remove_user_from_channel(
        &mut self,
        nick: &Nick,
        channel: &Channel,
    ) -> anyhow::Result<()> {
        if !self.writers.contains_key(nick) {
            panic!(
                "User {nick} does not already exist before being removed from channel {channel}"
            );
        }

        self.users_per_channel
            .entry(channel.clone())
            .or_insert(BTreeSet::new())
            .remove(nick);
        self.channels_per_user
            .entry(nick.clone())
            .or_insert(BTreeSet::new())
            .remove(channel);

        Ok(())
    }

    pub fn write(&mut self, target: &Target, message: &String) -> anyhow::Result<()> {
        match target {
            Target::User(nick) => self.write_to_user(nick, message),
            Target::Channel(channel) => self.write_to_channel(channel, message),
        }
    }

    pub fn write_to_user(&mut self, target: &Nick, message: &String) -> anyhow::Result<()> {
        match self.writers.get_mut(target) {
            Some(writer) => {
                writer
                    .lock()
                    .unwrap()
                    .write_message(format!("{}\r\n", message.trim_end()).as_str())?;
                Ok(())
            }
            None => Err(ErrorType::NoSuchNick),
        }
        .map_err(|e| anyhow!(e))
    }

    pub fn write_to_channel(&mut self, target: &Channel, message: &String) -> anyhow::Result<()> {
        let nicks = match self.users_per_channel.get(target) {
            Some(nicks) => Ok(nicks.clone()),
            None => Err(ErrorType::NoSuchChannel),
        }
        .map_err(|e| anyhow!(e))?;

        for nick in nicks {
            self.write_to_user(&nick, message)?;
        }

        Ok(())
    }

    pub fn write_to_users_channel(
        &mut self,
        target: &Nick,
        message: &String,
    ) -> anyhow::Result<()> {
        if let Some(channels) = self.channels_per_user.get(&target.clone()) {
            for channel in channels.clone().iter() {
                self.write_to_channel(&channel, message)?;
            }
        }

        Ok(())
    }
}
