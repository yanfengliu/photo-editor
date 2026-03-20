use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = dirs_data_path();
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("catalog.db");
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                format TEXT NOT NULL DEFAULT 'jpeg',
                width INTEGER NOT NULL DEFAULT 0,
                height INTEGER NOT NULL DEFAULT 0,
                date_taken TEXT,
                rating INTEGER NOT NULL DEFAULT 0,
                color_label TEXT NOT NULL DEFAULT 'none',
                flag TEXT NOT NULL DEFAULT 'none',
                camera TEXT,
                lens TEXT,
                iso INTEGER,
                focal_length REAL,
                aperture REAL,
                shutter_speed TEXT,
                thumbnail BLOB,
                edit_params TEXT,
                exif_json TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_images_date_taken ON images(date_taken);
            CREATE INDEX IF NOT EXISTS idx_images_rating ON images(rating);
            CREATE INDEX IF NOT EXISTS idx_images_color_label ON images(color_label);
            CREATE INDEX IF NOT EXISTS idx_images_flag ON images(flag);
            CREATE INDEX IF NOT EXISTS idx_images_camera ON images(camera);
            CREATE INDEX IF NOT EXISTS idx_images_file_path ON images(file_path);

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS image_tags (
                image_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (image_id, tag_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (parent_id) REFERENCES collections(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS collection_images (
                collection_id TEXT NOT NULL,
                image_id TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (collection_id, image_id),
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS edit_history (
                id TEXT PRIMARY KEY,
                image_id TEXT NOT NULL,
                action TEXT NOT NULL,
                params_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_edit_history_image ON edit_history(image_id, created_at);

            CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                image_id TEXT NOT NULL,
                name TEXT NOT NULL,
                params_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS presets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                params_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
        ")?;
        Ok(())
    }
}

fn dirs_data_path() -> PathBuf {
    if let Some(data_dir) = dirs_next_data() {
        data_dir.join("photo-editor")
    } else {
        PathBuf::from("./data")
    }
}

fn dirs_next_data() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        let db = Database { conn };
        db.create_tables().unwrap();
        db
    }

    #[test]
    fn test_create_database() {
        let db = make_test_db();
        // Verify tables exist
        let count: i32 = db.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='images'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_all_tables_created() {
        let db = make_test_db();
        let tables = ["images", "tags", "image_tags", "collections", "collection_images", "edit_history", "snapshots", "presets"];
        for table in &tables {
            let count: i32 = db.conn.query_row(
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'", table),
                [],
                |row| row.get(0),
            ).unwrap();
            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }

    #[test]
    fn test_insert_and_query_image() {
        let db = make_test_db();
        db.conn.execute(
            "INSERT INTO images (id, file_path, file_name, format, width, height) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["img-1", "/photos/test.jpg", "test.jpg", "jpeg", 1920, 1080],
        ).unwrap();

        let file_name: String = db.conn.query_row(
            "SELECT file_name FROM images WHERE id = 'img-1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(file_name, "test.jpg");
    }

    #[test]
    fn test_unique_file_path() {
        let db = make_test_db();
        db.conn.execute(
            "INSERT INTO images (id, file_path, file_name, format, width, height) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["img-1", "/photos/test.jpg", "test.jpg", "jpeg", 1920, 1080],
        ).unwrap();

        let result = db.conn.execute(
            "INSERT INTO images (id, file_path, file_name, format, width, height) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["img-2", "/photos/test.jpg", "test.jpg", "jpeg", 1920, 1080],
        );
        assert!(result.is_err(), "Duplicate file_path should fail");
    }

    #[test]
    fn test_default_values() {
        let db = make_test_db();
        db.conn.execute(
            "INSERT INTO images (id, file_path, file_name) VALUES (?1, ?2, ?3)",
            params!["img-1", "/photos/test.jpg", "test.jpg"],
        ).unwrap();

        let (rating, color_label, flag): (i32, String, String) = db.conn.query_row(
            "SELECT rating, color_label, flag FROM images WHERE id = 'img-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();

        assert_eq!(rating, 0);
        assert_eq!(color_label, "none");
        assert_eq!(flag, "none");
    }

    #[test]
    fn test_cascade_delete() {
        let db = make_test_db();
        db.conn.execute(
            "INSERT INTO images (id, file_path, file_name) VALUES (?1, ?2, ?3)",
            params!["img-1", "/photos/test.jpg", "test.jpg"],
        ).unwrap();
        db.conn.execute(
            "INSERT INTO edit_history (id, image_id, action, params_json) VALUES (?1, ?2, ?3, ?4)",
            params!["h-1", "img-1", "edit", "{}"],
        ).unwrap();

        db.conn.execute("DELETE FROM images WHERE id = 'img-1'", []).unwrap();

        let count: i32 = db.conn.query_row(
            "SELECT COUNT(*) FROM edit_history WHERE image_id = 'img-1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0, "Edit history should be cascade deleted");
    }

    #[test]
    fn test_tags_and_image_tags() {
        let db = make_test_db();
        db.conn.execute(
            "INSERT INTO images (id, file_path, file_name) VALUES (?1, ?2, ?3)",
            params!["img-1", "/photos/test.jpg", "test.jpg"],
        ).unwrap();
        db.conn.execute(
            "INSERT INTO tags (id, name) VALUES (?1, ?2)",
            params!["tag-1", "landscape"],
        ).unwrap();
        db.conn.execute(
            "INSERT INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
            params!["img-1", "tag-1"],
        ).unwrap();

        let tag_name: String = db.conn.query_row(
            "SELECT t.name FROM tags t JOIN image_tags it ON t.id = it.tag_id WHERE it.image_id = 'img-1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(tag_name, "landscape");
    }

    #[test]
    fn test_collections() {
        let db = make_test_db();
        db.conn.execute(
            "INSERT INTO collections (id, name) VALUES (?1, ?2)",
            params!["col-1", "Favorites"],
        ).unwrap();

        let name: String = db.conn.query_row(
            "SELECT name FROM collections WHERE id = 'col-1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(name, "Favorites");
    }

    #[test]
    fn test_indexes_exist() {
        let db = make_test_db();
        let indexes = [
            "idx_images_date_taken",
            "idx_images_rating",
            "idx_images_color_label",
            "idx_images_flag",
            "idx_images_camera",
            "idx_images_file_path",
        ];
        for idx in &indexes {
            let count: i32 = db.conn.query_row(
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='{}'", idx),
                [],
                |row| row.get(0),
            ).unwrap();
            assert_eq!(count, 1, "Index {} should exist", idx);
        }
    }
}
