use super::Database;

pub struct SkinData {
    pub data: Vec<u8>,
    pub mime: String,
}

impl Database {
    pub fn set_skin(&self, user_id: i64, data: &[u8], mime: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET skin_blob = ?1, skin_mime = ?2 WHERE id = ?3",
            rusqlite::params![data, mime, user_id],
        )
        .map_err(|e| format!("Failed to save skin: {}", e))?;
        Ok(())
    }

    pub fn get_skin(&self, user_id: i64) -> Option<SkinData> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT skin_blob, skin_mime FROM users WHERE id = ?1 AND skin_blob IS NOT NULL",
            rusqlite::params![user_id],
            |row| {
                Ok(SkinData {
                    data: row.get(0)?,
                    mime: row.get(1)?,
                })
            },
        )
        .ok()
    }

    pub fn has_skin(&self, user_id: i64) -> bool {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM users WHERE id = ?1 AND skin_blob IS NOT NULL",
            rusqlite::params![user_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
            > 0
    }
}
