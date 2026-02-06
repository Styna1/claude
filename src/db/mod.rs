pub mod schema;
pub mod accounts;
pub mod skins;

use rusqlite::Connection;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("Failed to open database");
        let db = Database {
            conn: Mutex::new(conn),
        };
        schema::initialize(&db);
        db
    }
}
