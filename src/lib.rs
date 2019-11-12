#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;

pub fn establish_connection() -> MysqlConnection {
    let database_url = "mysql://user:password@localhost:3306/page_monitor";

    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}