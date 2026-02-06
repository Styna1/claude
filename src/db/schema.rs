use super::Database;

pub fn initialize(db: &Database) {
    let conn = db.conn.lock().unwrap();
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            username      TEXT    NOT NULL UNIQUE,
            password_hash TEXT    NOT NULL,
            skin_blob     BLOB,
            skin_mime     TEXT,
            created_at    TEXT    DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sessions (
            token      TEXT    PRIMARY KEY,
            user_id    INTEGER NOT NULL REFERENCES users(id),
            expires_at TEXT    NOT NULL
        );
        ",
    )
    .expect("Failed to initialize database schema");
}
