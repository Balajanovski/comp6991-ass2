use crate::connect::ConnectionWrite;
use crate::types::{Nick, ErrorType};
use concurrent_hashmap::ConcHashMap;

pub struct UserConnections {
    /// A bit of design excellence
    /// Utilising a concurrent hashmap to reduce lock contention
    writers: ConcHashMap<String, ConnectionWrite>,
}

impl UserConnections {
    pub fn new() -> UserConnections {
        UserConnections { writers: ConcHashMap::<String, ConnectionWrite>::new() }
    }

    pub fn add_user(&mut self, nick: &Nick, conn_write: ConnectionWrite) -> Result<(), ErrorType> {
        let nick = nick.to_string();
        if let Some(_) = self.writers.insert(nick, conn_write) {
            Err(ErrorType::NickCollision)
        } else {
            Ok(())
        }
    }

    pub fn remove_user(&mut self, nick: &Nick) {
        self.writers.remove(&nick.to_string());
    }

    pub fn write_to_user(&self, target: &Nick, message: &str) -> Result<(), ErrorType> {
        match self.writers.find_mut(&target.to_string()) {
            Some(mut writer) => {
                writer.get().write_message(message);
                Ok(())
            }
            None => Err(ErrorType::NoSuchNick),
        }
    }
}