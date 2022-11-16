use crate::connect::ConnectionWrite;
use crate::types::{Nick, ErrorType, Reply, Channel};
use std::collections::{BTreeMap, BTreeSet};

pub struct UserConnections {
    channel_per_user: BTreeMap<Nick, Channel>,
    users_per_channel: BTreeMap<Channel, BTreeSet<Nick>>,
    writers: BTreeMap<Nick, ConnectionWrite>,
}

impl UserConnections {
    pub fn new() -> UserConnections {
        UserConnections { 
            users_per_channel: BTreeMap::new(),
            channel_per_user: BTreeMap::new(),
            writers: BTreeMap::new(), 
        }
    }

    pub fn add_user(&mut self, nick: &Nick, conn_write: ConnectionWrite) -> Result<(), ErrorType> {
        if let Some(_) = self.writers.insert(nick.clone(), conn_write) {
            Err(ErrorType::NickCollision)
        } else {
            Ok(())
        }
    }

    pub fn remove_user(&mut self, nick: &Nick) {
        self.writers.remove(nick);
        if let Some(channel) = self.channel_per_user.get(nick) {
            self.users_per_channel.get_mut(channel).map(|nicks| nicks.remove(nick));
            self.channel_per_user.remove(nick);
        }
    }

    pub fn add_user_to_channel(&mut self, nick: &Nick, channel: &Channel) -> Result<(), ErrorType> {
        if !self.writers.contains_key(nick) {
            panic!("User does not already exist before being added to channel");
        }

        self.users_per_channel.entry(channel.clone()).or_insert(BTreeSet::new()).insert(nick.clone());
        self.channel_per_user.insert(nick.clone(), channel.clone());

        Ok(())
    }

    pub fn write_to_user(&mut self, target: &Nick, message: &Reply) -> Result<(), ErrorType> {
        match self.writers.get_mut(target) {
            Some(writer) => {
                writer.write_message(message.to_string().as_str());
                Ok(())
            },
            None => Err(ErrorType::NoSuchNick),
        }
    }

    pub fn write_to_channel(&mut self, target: &Channel, message: &Reply) -> Result<(), ErrorType> {
        let nicks = match self.users_per_channel.get(target) {
            Some(nicks) => Ok(nicks.clone()),
            None => Err(ErrorType::NoSuchChannel),
        };

        match nicks {
            Ok(nicks) => {
                for nick in nicks {
                    self.write_to_user(&nick, message);
                }
                Ok(())
            },
            Err(err) => {
                Err(err)
            }
        }
    }

    pub fn write_to_users_channel(&mut self, target: &Nick, message: &Reply) -> Result<(), ErrorType> {
        if let Some(channel) = self.channel_per_user.get(target) {
            self.write_to_channel(&channel.clone(), message)
        } else {
            Ok(())
        }
    }
}