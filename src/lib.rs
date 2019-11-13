#[macro_use]
extern crate diesel;
#[macro_use]
extern crate dotenv_codegen;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;

pub fn establish_connection() -> MysqlConnection {
    println!("{}", dotenv!("DATABASE_URL"));
    let database_url = dotenv!("DATABASE_URL");

    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}