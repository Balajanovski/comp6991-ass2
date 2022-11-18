//! A utility IrcClient for integration tests
//! Based off of ircat: https://github.com/COMP6991UNSW/ircat/blob/main/src/main.rs

use anyhow::anyhow;
use bufstream::BufStream;
use closure::closure;
use log::{error, info};
use std::io::{BufRead, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct IrcClient {
    stream_write: TcpStream,
    resp_receiver: Receiver<String>,
    quit_flag: Arc<AtomicBool>,
}

impl IrcClient {
    pub fn new(ip: IpAddr, port: u16) -> IrcClient {
        let stream_read = TcpStream::connect((ip, port))
            .unwrap_or_else(|_| panic!("failed to connect to {ip}:{port}"));
        let stream_write = stream_read.try_clone().expect("failed to clone connection");
        let stream_read = BufStream::new(stream_read);
        let quit_flag = Arc::new(AtomicBool::new(false));
        let thread_quit_flag = quit_flag.clone();
        let (resp_sender, resp_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

        {
            thread::spawn(
                closure!(move thread_quit_flag, move mut stream_read, move resp_sender, || {
                    while !thread_quit_flag.load(Ordering::Relaxed) {
                        let mut line = String::new();
                        match stream_read.read_line(&mut line) {
                            Ok(0) => {
                                info!("IrcClient EOF");
                                thread_quit_flag.store(true, Ordering::Relaxed);
                            }
                            Err(err) => {
                                error!("Error while IrcClient read {err}");
                                thread_quit_flag.store(true, Ordering::Relaxed);
                            }
                            Ok(_) => {
                                let line = line.trim();
                                let _ = resp_sender.send(line.to_string());
                            }
                        }
                    }
                }),
            );
        }

        IrcClient {
            stream_write,
            resp_receiver,
            quit_flag,
        }
    }

    pub fn send_message(&mut self, msg: &String) {
        if let Err(err) = {
            self.stream_write
                .write_all(format!("{}\r\n", msg.trim_end()).as_bytes())
                .and_then(|_| self.stream_write.flush())
        } {
            error!("Error while IrcClient sent {err}");
        };
    }

    pub fn get_message(&self) -> anyhow::Result<String> {
        self.resp_receiver
            .recv_timeout(Duration::from_secs(10))
            .map_err(|e| anyhow!(e))
    }
}

impl Drop for IrcClient {
    fn drop(&mut self) {
        // Let the threads know to clean up
        self.quit_flag.store(true, Ordering::Relaxed);
    }
}
