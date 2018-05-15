pub extern crate irc as irc_crate;
pub use self::irc_crate::client::prelude::*;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub enum InternalThreadMsg {
    JOIN_CHANNEL(String)
}
use self::InternalThreadMsg::*;

pub struct IRCService {
    internal_sender: Sender<InternalThreadMsg>,
    message_receiver: Receiver<Message>
}

impl IRCService {
    pub fn start_service() -> IRCService {
        let config = Config {
            nickname: Some("Nico_Scarlet".to_owned()),
            server: Some("irc.chat.twitch.tv".to_owned()),
		    port: Some(6667),
		    use_ssl: Some(false),
            ..Config::default()
        };
        
        let (internal_sender, internal_receiver) = mpsc::channel();
        let (message_sender, message_receiver) = mpsc::channel();

        let _self = IRCService { internal_sender, message_receiver };

        thread::spawn(move || {
            let mut reactor = IrcReactor::new().unwrap();
            let client = reactor.prepare_client_and_connect(&config).unwrap();
		    client.send(Command::PASS("oauth:bvfkov2tepy3jfb3fcxk2ezwx91erw".to_owned())).expect("Failed to send PASS");
            client.send(Command::NICK("nico_scarlet".to_owned())).expect("Failed to send NICK.");

            reactor.register_client_with_handler(client, move |client, message| {
                while let Some(message) = internal_receiver.try_recv().ok() {
                    match message {
                        JOIN_CHANNEL(channel) => { println!("Joining channel {}", channel); client.send(Command::JOIN(channel, None, None)).expect("Failed to send JOIN command"); }
                        _ => {}
                    }
                }
                
                message_sender.send(message).ok();

                Ok(())
            });

            reactor.run().unwrap();
        });
        
        _self
    }

    pub fn try_poll_message(&self) -> Option<Message> {
        self.message_receiver.try_recv().ok()
    }

    pub fn join_channel(&self, channel: String) {
        self.internal_sender.send(JOIN_CHANNEL(channel)).unwrap();
    }
}


