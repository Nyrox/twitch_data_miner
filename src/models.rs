extern crate mysql;

use mysql as my;
use my::prelude::*;


/*
 * CHANNELS
 * */
pub struct ModelChannels {}

#[derive(Clone, Debug)]
pub struct Channel {
    pub id: i64,
    pub login: String
}

impl ModelChannels {
    pub fn create_tables<Conn>(conn: &mut Conn) where Conn: GenericConnection {
        conn.prep_exec(r"
            CREATE TABLE IF NOT EXISTS Channels(
                id INT PRIMARY KEY NOT NULL,
                login VARCHAR(32) NOT NULL
            )
        ", ()).unwrap();
    }

    // Seriously rewrite this
    pub fn get_single<Conn>(conn: &mut Conn, id: i64) -> Option<Channel> where Conn: GenericConnection {
        let results = conn.prep_exec(r"
            SELECT id, login FROM Channels WHERE id=:id
        ", (params!{"id" => id})).map(|result|{
            result.map(|x| x.unwrap()).map(|row| {
                let (id, login) = my::from_row(row);
                Channel { id, login }
            }).collect::<Vec<Channel>>()
        }).unwrap();

        results.get(0).map(|v| v.clone())
    }
    
    // Inserts a record into the db.
    pub fn insert<Conn>(conn: &mut Conn, dataset: Channel) where Conn: GenericConnection {
        conn.prep_exec(r"
            INSERT INTO Channels
                VALUES(:id, :login);
        ", params!{ 
            "id" => dataset.id,
            "login" => dataset.login
        });

    }
}


