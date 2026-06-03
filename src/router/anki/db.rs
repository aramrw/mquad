use crate::YomichanApp;

#[derive(Debug, Clone)]
pub struct QueuedCard {
    pub id: i64,
    pub headword: String,
    pub reading: String,
    pub definition: String,
    pub sentence: String,
    pub added_at: i64,
}

impl YomichanApp {
    pub fn init_anki_queue_db(&self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS anki_queue (
                id INTEGER PRIMARY KEY,
                headword TEXT,
                reading TEXT,
                definition TEXT,
                sentence TEXT,
                added_at INTEGER
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_anki_queue(&self, headword: &str, reading: &str, definition: &str, sentence: &str) -> Result<(), rusqlite::Error> {
        self.init_anki_queue_db()?;
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        conn.execute(
            "INSERT INTO anki_queue (headword, reading, definition, sentence, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![headword, reading, definition, sentence, now],
        )?;
        Ok(())
    }

    pub fn get_anki_queue(&self) -> Result<Vec<QueuedCard>, rusqlite::Error> {
        self.init_anki_queue_db()?;
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        let mut stmt = conn.prepare("SELECT id, headword, reading, definition, sentence, added_at FROM anki_queue ORDER BY added_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(QueuedCard {
                id: row.get(0)?,
                headword: row.get(1)?,
                reading: row.get(2)?,
                definition: row.get(3)?,
                sentence: row.get(4)?,
                added_at: row.get(5)?,
            })
        })?;

        let mut cards = Vec::new();
        for row in rows {
            cards.push(row?);
        }
        Ok(cards)
    }

    pub fn delete_anki_queue_item(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open("yomichan_rs/db.ycd")?;
        conn.execute("DELETE FROM anki_queue WHERE id = ?1", [id])?;
        Ok(())
    }
}