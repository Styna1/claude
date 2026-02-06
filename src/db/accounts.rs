use super::Database;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use uuid::Uuid;
use crate::config::SESSION_EXPIRY_HOURS;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
}

impl Database {
    pub fn register(&self, username: &str, password: &str) -> Result<User, String> {
        if username.len() < 2 || username.len() > 20 {
            return Err("Username must be 2-20 characters".into());
        }
        if password.len() < 4 {
            return Err("Password must be at least 4 characters".into());
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Hash error: {}", e))?
            .to_string();

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            rusqlite::params![username, hash],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                "Username already taken".to_string()
            } else {
                format!("Database error: {}", e)
            }
        })?;

        let id = conn.last_insert_rowid();
        Ok(User {
            id,
            username: username.to_string(),
        })
    }

    pub fn login(&self, username: &str, password: &str) -> Result<(User, String), String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, username, password_hash FROM users WHERE username = ?1")
            .map_err(|e| format!("DB error: {}", e))?;

        let user_row = stmt
            .query_row(rusqlite::params![username], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|_| "Invalid username or password".to_string())?;

        let (id, uname, hash_str) = user_row;

        let parsed_hash =
            PasswordHash::new(&hash_str).map_err(|e| format!("Hash parse error: {}", e))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| "Invalid username or password".to_string())?;

        let token = Uuid::new_v4().to_string();
        let expires = Utc::now() + Duration::hours(SESSION_EXPIRY_HOURS);
        let expires_str = expires.to_rfc3339();

        conn.execute(
            "INSERT INTO sessions (token, user_id, expires_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![token, id, expires_str],
        )
        .map_err(|e| format!("Session error: {}", e))?;

        Ok((
            User {
                id,
                username: uname,
            },
            token,
        ))
    }

    pub fn validate_session(&self, token: &str) -> Option<User> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.query_row(
            "SELECT u.id, u.username FROM sessions s JOIN users u ON s.user_id = u.id WHERE s.token = ?1 AND s.expires_at > ?2",
            rusqlite::params![token, now],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                })
            },
        )
        .ok()
    }

    pub fn logout(&self, token: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM sessions WHERE token = ?1", rusqlite::params![token]);
    }

    pub fn get_user_by_id(&self, user_id: i64) -> Option<User> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, username FROM users WHERE id = ?1",
            rusqlite::params![user_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                })
            },
        )
        .ok()
    }
}
