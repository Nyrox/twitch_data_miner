#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate mysql;
extern crate curl;

extern crate rocket;


use curl::easy::{Easy, List};

use mysql as my;
use my::prelude::*;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
use serde::de::{self, Deserializer, Visitor, Unexpected}; 

mod twitch;
use twitch::TwitchAPI;

mod models;
use models::ModelChannels;

mod irc;
use irc::IRCService;
use irc::irc_crate::client::prelude::*;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::marker::PhantomData;
use std::fmt;
use std::str;

struct ParseVisitor<T> { 
    _marker: PhantomData<T>   
}

impl<'de, T> Visitor<'de> for ParseVisitor<T>
    where T: std::str::FromStr 
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string object that parses to T")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where E: de::Error,
    {
        if let Ok(t) = s.parse::<T>() {
            Ok(t)
        }
        else {
            Err(de::Error::invalid_value(Unexpected::Str(s), &self))
        }
    }
}

fn deserialize_parse<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where D: Deserializer<'de>, T: str::FromStr 
{
    deserializer.deserialize_string(ParseVisitor::<T> { _marker: PhantomData })      
}

#[derive(Debug, Deserialize)]
struct Stream {
    #[serde(deserialize_with="deserialize_parse")]
    id: u64,
    #[serde(deserialize_with="deserialize_parse")]
    user_id: u64,
    #[serde(deserialize_with="deserialize_parse")]
    game_id: u64,
    title: String,
}

fn get_streams(api: &mut TwitchAPI, count: i32) -> Vec<Stream> {
    return serde_json::from_value(
        api.request()
            .resource("streams".to_owned())
            .param(("first".to_owned(), format!("{}", count)))
            .get()["data"].take()
    ).expect("Failed to parse stream response to json.");
}


// The controller is responsible for managing the observers and regularily updating the client
// state
struct Controller {
    irc_service: IRCService,
    api: TwitchAPI,
    sql_pool: my::Pool
}

impl Controller {
    pub fn execute() -> ! {
        let mut api = TwitchAPI::new("bvfkov2tepy3jfb3fcxk2ezwx91erw".to_owned());
        let mut sql_pool = my::Pool::new(get_options()).unwrap();

        let mut _self = Controller { 
            api: TwitchAPI::new("bvfkov2tepy3jfb3fcxk2ezwx91erw".to_owned()),
            irc_service: IRCService::start_service(),
            sql_pool: my::Pool::new(get_options()).unwrap()
        };
        
        let streams = get_streams(&mut api, 20);
        println!("{:?}", streams);

 /*    // Grab the top 100 channels
        let data = _self.api.request()
            .resource("streams".to_owned())
            .param(("first".to_owned(), format!("{}", 20)))
            .get();
 
        println!("{}", data);
    
        let mut channels = Vec::<String>::new();
        
        println!("Polling channels...");
        for e in data["data"].as_array().expect("data not a valid array") {            
            let channel = ModelChannels::get_single(&mut _self.get_conn(), e["user_id"].as_str().unwrap().parse::<i64>().expect("Failed to parse user_id into integer"));
            
            let channel = match channel {
                Some(c) => c,
                None => {
                    println!("Channel not listed in db. Fetching from twitch");
                    let user = &_self.api.request()
                        .resource("users".to_owned())
                        .param(("id".to_owned(), e["user_id"].as_str().expect("1").to_owned()))
                        .get()["data"][0];

                     let channel = models::Channel { id: user["id"].as_str().unwrap().parse().unwrap(), login: user["login"].as_str().unwrap().to_owned() };
                     ModelChannels::insert(&mut _self.get_conn(), channel.clone());
                     channel
                }
            };

            channels.push(format!("#{}", channel.login.clone()));
        }
        for channel in channels {
            _self.irc_service.join_channel(channel);
        }
 */   
        println!("Listening for messages now: ");
        loop {
            if let Some(message) = _self.irc_service.try_poll_message() {
                _self.handle_message(message);
            }
        }
    }

    pub fn get_conn(&mut self) -> my::PooledConn {
        self.sql_pool.get_conn().expect("Failed to get SQL Connection from Pool")
    }

    pub fn handle_message(&mut self, message: Message) {
        
        match &message.command {
            &Command::PRIVMSG(ref target, ref content) => { 
                let login = target.split_at(1).1;
                let user = message.source_nickname();
                let mut conn = self.get_conn();

                conn.prep_exec(r"
                    UPDATE Channels SET message_count=message_count + 1 WHERE login=:channel
                ", params!{ "channel" => login }).expect("Failed to increment message count on channel.");

                if let Some(user) = user {
                    conn.prep_exec(r"
                        UPDATE Users SET message_count=message_count + 1 WHERE nick=:nick
                    ", params!{ "nick" => user }).expect("Failed to increment message count on user.");
                }
            },
            _ => {}
        }

    }

}

#[get("/")]
fn index() -> &'static str {
    "Hello World"
}

fn main() {
    Controller::execute();

	// drop_tables(&mut pool.get_conn().expect("Failed to get connection."));
	// create_tables(&mut pool.get_conn().expect("Failed to get connection."));
}

fn get_options() -> my::Opts {
	let mut builder = my::OptsBuilder::new();
	builder
		.user(Some("twitch_user"))
        .pass(Some("letme1n"))
		.ip_or_hostname(Some("localhost"))
		.db_name(Some("twitch_data_mining"));
		//.prefer_socket(false);

	return my::Opts::from(builder);
}

fn drop_tables<T>(conn: &mut T)
	where T: GenericConnection {

	conn.prep_exec(r"
		DROP TABLE Users
	", ()).unwrap();
}

fn create_tables<T>(conn: &mut T)
 	where T: GenericConnection {

	conn.prep_exec(r"
		CREATE TABLE Users(
			id INT PRIMARY KEY NOT NULL,
			username VARCHAR(32) NOT NULL
		)
	", ()).unwrap();
}
