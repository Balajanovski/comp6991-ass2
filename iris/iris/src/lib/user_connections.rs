use crate::connect::ConnectionWrite;
use crate::types::{Nick, ErrorType, Reply};
use dashmap::DashMap;

pub struct UserConnections {
    /// A bit of design excellence
    /// Utilising a concurrent hashmap to reduce lock contention
    writers: DashMap<String, ConnectionWrite>,
}

impl UserConnections {
    pub fn new() -> UserConnections {
        UserConnections { writers: DashMap::new() }
    }

    pub fn add_user(&self, nick: &Nick, conn_write: ConnectionWrite) -> Result<(), ErrorType> {
        let nick = nick.to_string();
        if let Some(_) = self.writers.insert(nick, conn_write) {
            Err(ErrorType::NickCollision)
        } else {
            Ok(())
        }
    }

    pub fn remove_user(&self, nick: &Nick) {
        self.writers.remove(&nick.to_string());
    }

    pub fn write_to_user(&self, target: &Nick, message: &Reply) -> Result<(), ErrorType> {
        match self.writers.get_mut(&target.to_string()) {
            Some(mut writer) => {
                writer.write_message(message.to_string().as_str());
                Ok(())
            }
            None => Err(ErrorType::NoSuchNick),
        }
    }
}