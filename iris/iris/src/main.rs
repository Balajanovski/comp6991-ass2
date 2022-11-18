#![feature(scoped_threads)]

#[macro_use]
extern crate log;
extern crate simplelog;

mod message_handler;
mod plugin_handler;
mod user_connections;

use crate::{message_handler::MessageHandler, user_connections::UserConnections};
use anyhow::anyhow;
use clap::Parser;
use common::{connect::ConnectionManager, types::SERVER_NAME};
use simplelog::*;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Parser)]
struct Arguments {
    #[clap(default_value = "127.0.0.1")]
    ip_address: IpAddr,

    #[clap(default_value = "6991")]
    port: u16,

    #[clap(long)]
    plugins: Vec<String>,
}

fn main() {
    let arguments = Arguments::parse();
    begin_server(&arguments.ip_address, arguments.port, &arguments.plugins);
}

fn begin_server(ip_address: &IpAddr, port: u16, plugins: &Vec<String>) {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    info!("Launching {} at {}:{}", SERVER_NAME, ip_address, port,);

    let mut connection_manager = ConnectionManager::launch(ip_address.clone(), port.clone());
    let user_connections = Arc::new(Mutex::new(UserConnections::new()));

    thread::scope(|s| {
        loop {
            // This function call will block until a new client connects!
            let (mut conn_read, conn_write) = connection_manager.accept_new_connection();
            let thread_user_connections = user_connections.clone();
            let thread_plugin_list = plugins.clone();
            info!("New connection from {}", conn_read.id());

            s.spawn(move || {
                let mut handler =
                    MessageHandler::new(&thread_user_connections, conn_write, thread_plugin_list);
                while !handler.has_quit() {
                    info!("Waiting for message...");

                    let message = conn_read.read_message();
                    handler.handle(message.map_err(|e| anyhow!(e)));
                }

                info!("Connection has closed...");
            });
        }
    });
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use common::irc_client::IrcClient;
    use std::{net::Ipv4Addr, time::Duration};

    static IP_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    const PORT: u16 = 6991;
    static PLUGINS: Vec<String> = vec![];

    #[test]
    fn test_flow() {
        let mut client = initialise_test_rig();

        // Error handling in nicknames (and ignoring other commands)
        client.send_message(&"PING :me".to_string());
        client.send_message(&"NOCK".to_string());
        assert_eq!(
            ":iris-server 421 :Unknown command".to_string(),
            client.get_message().unwrap()
        );
        client.send_message(&"NICK wiz".to_string());

        // Error handling in usernames (and ignoring other commands)
        client.send_message(&"PING :me".to_string());
        client.send_message(&"USERR ignored ignored ignored :Ronnie Reagan".to_string());
        assert_eq!(
            ":iris-server 421 :Unknown command".to_string(),
            client.get_message().unwrap()
        );
        client.send_message(&"USER ignored ignored ignored :Ronnie Reagan".to_string());
        assert_eq!(
            ":iris-server 001 wiz :Hi Ronnie Reagan, welcome to IRC",
            client.get_message().unwrap()
        );

        // Ping
        client.send_message(&"PING :me".to_string());
        assert_eq!("PONG :me".to_string(), client.get_message().unwrap());

        // Message self
        client.send_message(&"PRIVMSG wiz :hi".to_string());
        assert_eq!(
            ":wiz PRIVMSG wiz :hi".to_string(),
            client.get_message().unwrap()
        );

        // Channels
        // When you join a channel:
        client.send_message(&"JOIN #channel".to_string());
        // You should get notification of your own join
        assert_eq!(
            ":wiz JOIN #channel".to_string(),
            client.get_message().unwrap()
        );
        // You should see your own message to the channel
        client.send_message(&"PRIVMSG #channel :hello".to_string());
        assert_eq!(
            ":wiz PRIVMSG #channel :hello".to_string(),
            client.get_message().unwrap()
        );
        // After departing, you shouldn't see channel messages
        client.send_message(&"PART #channel".to_string());
        client.send_message(&"PRIVMSG #channel :hello".to_string());
        client.send_message(&"PING :me".to_string());
        assert_eq!("PONG :me".to_string(), client.get_message().unwrap());
    }

    fn initialise_test_rig() -> IrcClient {
        thread::spawn(|| {
            begin_server(&IP_ADDR, PORT, &PLUGINS);
        });

        // Having timing in tests is bad
        // However, I'm too lazy to refactor this to have
        // a proper integration testing rig
        thread::sleep(Duration::from_secs(1));
        IrcClient::new(IP_ADDR, PORT)
    }
}
