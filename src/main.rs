extern crate irc;

#[macro_use]
extern crate mysql;
extern crate curl;

use curl::easy::{Easy, List};

use mysql as my;
use my::prelude::*;

use irc::client::prelude::*;

extern crate serde;
extern crate serde_json;

mod twitch;
use twitch::TwitchAPI;

fn main() {
    let observer_count = 10;

    let mut api = TwitchAPI::new("bvfkov2tepy3jfb3fcxk2ezwx91erw".to_owned());
    let data = api.generic_request("https://api.twitch.tv/helix/streams");
    println!("{}", data);
    
    let channels = Vec::<String>::new();

    

	let entries = &data["data"];
    
    // Generate list of channels
    let channels = Vec::<String>::new();
    for e in entries.as_array().expect("Data is not a valid array") {
        //println!("{:?}", e);
        (|| {
            let mut easy = Easy::new();
            easy.url("https://api.twitch.tv/helix/users/?id=15564828&id=100814397").unwrap();
            let mut headers = List::new();
	        headers.append("Authorization: Bearer bvfkov2tepy3jfb3fcxk2ezwx91erw");
	        easy.http_headers(headers).expect("failed to set headers");
            let mut dst = Vec::new();

            {
                let mut transfer = easy.transfer();
                transfer.write_function(|data| { dst.extend_from_slice(data); Ok(data.len()) }).unwrap();
                transfer.perform().unwrap();
            }
            println!("{}", &String::from_utf8(dst).unwrap());
        })();
    }
    return;

    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("Nico_Scarlet".to_owned()),
        server: Some("irc.chat.twitch.tv".to_owned()),
		port: Some(6667),
		use_ssl: Some(false),
        channels: Some(vec!["#39298218".to_owned()]),
        ..Config::default()
    };


    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
		client.send(Command::PASS("oauth:bvfkov2tepy3jfb3fcxk2ezwx91erw".to_owned()));
    client.send(Command::NICK("nico_scarlet".to_owned()));


    reactor.register_client_with_handler(client, |client, message| {
        print!("{}", message);
        // And here we can do whatever we want with the messages.
        Ok(())
    });

    reactor.run().unwrap();
	

    return;

	let pool = my::Pool::new(get_options()).unwrap();

	// drop_tables(&mut pool.get_conn().expect("Failed to get connection."));
	// create_tables(&mut pool.get_conn().expect("Failed to get connection."));

}

fn get_options() -> my::Opts {
	let mut builder = my::OptsBuilder::new();
	builder
		.user(Some("root"))
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
