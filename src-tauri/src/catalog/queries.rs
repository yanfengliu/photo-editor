use rusqlite::params;
use crate::catalog::db::Database;
use crate::catalog::models::*;

pub fn get_images(
    db: &Database,
    offset: u32,
    limit: u32,
    sort_by: &str,
    sort_order: &str,
) -> Result<Vec<ImageRecord>, Box<dyn std::error::Error>> {
    let allowed_sorts = ["date_taken", "rating", "file_name", "created_at"];
    let sort_col = if allowed_sorts.contains(&sort_by) { sort_by } else { "date_taken" };
    let order = if sort_order.eq_ignore_ascii_case("ASC") { "ASC" } else { "DESC" };

    let query = format!(
        "SELECT id, file_path, file_name, format, width, height, date_taken, rating,
                color_label, flag, camera, lens, iso, focal_length, aperture, shutter_speed,
                edit_params, created_at
         FROM images ORDER BY {} {} LIMIT ? OFFSET ?",
        sort_col, order
    );

    let mut stmt = db.conn.prepare(&query)?;
    let rows = stmt.query_map(params![limit, offset], |row| {
        Ok(ImageRecord {
            id: row.get(0)?,
            file_path: row.get(1)?,
            file_name: row.get(2)?,
            format: row.get(3)?,
            width: row.get(4)?,
            height: row.get(5)?,
            date_taken: row.get(6)?,
            rating: row.get(7)?,
            color_label: row.get(8)?,
            flag: row.get(9)?,
            camera: row.get(10)?,
            lens: row.get(11)?,
            iso: row.get(12)?,
            focal_length: row.get(13)?,
            aperture: row.get(14)?,
            shutter_speed: row.get(15)?,
            edit_params: row.get(16)?,
            tags: Vec::new(),
            created_at: row.get(17)?,
        })
    })?;

    let mut images: Vec<ImageRecord> = Vec::new();
    for row in rows {
        let mut img = row?;
        img.tags = get_tags_for_image(db, &img.id)?;
        images.push(img);
    }
    Ok(images)
}

pub fn get_image_by_id(
    db: &Database,
    image_id: &str,
) -> Result<ImageRecord, Box<dyn std::error::Error>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, file_path, file_name, format, width, height, date_taken, rating,
                color_label, flag, camera, lens, iso, focal_length, aperture, shutter_speed,
                edit_params, created_at
         FROM images WHERE id = ?1"
    )?;

    let img = stmt.query_row(params![image_id], |row| {
        Ok(ImageRecord {
            id: row.get(0)?,
            file_path: row.get(1)?,
            file_name: row.get(2)?,
            format: row.get(3)?,
            width: row.get(4)?,
            height: row.get(5)?,
            date_taken: row.get(6)?,
            rating: row.get(7)?,
            color_label: row.get(8)?,
            flag: row.get(9)?,
            camera: row.get(10)?,
            lens: row.get(11)?,
            iso: row.get(12)?,
            focal_length: row.get(13)?,
            aperture: row.get(14)?,
            shutter_speed: row.get(15)?,
            edit_params: row.get(16)?,
            tags: Vec::new(),
            created_at: row.get(17)?,
        })
    })?;

    Ok(img)
}

pub fn get_thumbnail(
    db: &Database,
    image_id: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut stmt = db.conn.prepare("SELECT thumbnail FROM images WHERE id = ?1")?;
    let thumbnail: Vec<u8> = stmt.query_row(params![image_id], |row| row.get(0))?;
    Ok(thumbnail)
}

pub fn set_rating(
    db: &Database,
    image_id: &str,
    rating: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let rating = rating.min(5);
    db.conn.execute(
        "UPDATE images SET rating = ?1 WHERE id = ?2",
        params![rating, image_id],
    )?;
    Ok(())
}

pub fn set_color_label(
    db: &Database,
    image_id: &str,
    color_label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    db.conn.execute(
        "UPDATE images SET color_label = ?1 WHERE id = ?2",
        params![color_label, image_id],
    )?;
    Ok(())
}

pub fn set_flag(
    db: &Database,
    image_id: &str,
    flag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    db.conn.execute(
        "UPDATE images SET flag = ?1 WHERE id = ?2",
        params![flag, image_id],
    )?;
    Ok(())
}

pub fn add_tags(
    db: &Database,
    image_id: &str,
    tags: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    for tag_name in tags {
        let tag_id = uuid::Uuid::new_v4().to_string();
        db.conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
            params![tag_id, tag_name],
        )?;

        let actual_tag_id: String = db.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag_name],
            |row| row.get(0),
        )?;

        db.conn.execute(
            "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
            params![image_id, actual_tag_id],
        )?;
    }
    Ok(())
}

pub fn remove_tag(
    db: &Database,
    image_id: &str,
    tag_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    db.conn.execute(
        "DELETE FROM image_tags WHERE image_id = ?1 AND tag_id = (SELECT id FROM tags WHERE name = ?2)",
        params![image_id, tag_name],
    )?;
    Ok(())
}

pub fn get_tags_for_image(
    db: &Database,
    image_id: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut stmt = db.conn.prepare(
        "SELECT t.name FROM tags t JOIN image_tags it ON t.id = it.tag_id WHERE it.image_id = ?1"
    )?;
    let rows = stmt.query_map(params![image_id], |row| row.get(0))?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn create_collection(
    db: &Database,
    name: &str,
    parent_id: Option<&str>,
) -> Result<CollectionRecord, Box<dyn std::error::Error>> {
    let id = uuid::Uuid::new_v4().to_string();
    db.conn.execute(
        "INSERT INTO collections (id, name, parent_id) VALUES (?1, ?2, ?3)",
        params![id, name, parent_id],
    )?;
    Ok(CollectionRecord {
        id,
        name: name.to_string(),
        parent_id: parent_id.map(String::from),
        image_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub fn add_to_collection(
    db: &Database,
    collection_id: &str,
    image_ids: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    for (i, image_id) in image_ids.iter().enumerate() {
        db.conn.execute(
            "INSERT OR IGNORE INTO collection_images (collection_id, image_id, position) VALUES (?1, ?2, ?3)",
            params![collection_id, image_id, i as i32],
        )?;
    }
    Ok(())
}

pub fn get_collections(
    db: &Database,
) -> Result<Vec<CollectionRecord>, Box<dyn std::error::Error>> {
    let mut stmt = db.conn.prepare(
        "SELECT c.id, c.name, c.parent_id, c.created_at,
                (SELECT COUNT(*) FROM collection_images ci WHERE ci.collection_id = c.id) as img_count
         FROM collections c ORDER BY c.name"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CollectionRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            created_at: row.get(3)?,
            image_count: row.get(4)?,
        })
    })?;
    let mut collections = Vec::new();
    for row in rows {
        collections.push(row?);
    }
    Ok(collections)
}

pub fn delete_images(
    db: &Database,
    image_ids: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    for image_id in image_ids {
        db.conn.execute("DELETE FROM images WHERE id = ?1", params![image_id])?;
    }
    Ok(())
}

pub fn save_edit_params(
    db: &Database,
    image_id: &str,
    params_json: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    db.conn.execute(
        "UPDATE images SET edit_params = ?1 WHERE id = ?2",
        params![params_json, image_id],
    )?;

    // Also save to history
    let history_id = uuid::Uuid::new_v4().to_string();
    db.conn.execute(
        "INSERT INTO edit_history (id, image_id, action, params_json) VALUES (?1, ?2, 'edit', ?3)",
        params![history_id, image_id, params_json],
    )?;

    Ok(())
}

pub fn save_snapshot(
    db: &Database,
    image_id: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_params: Option<String> = db.conn.query_row(
        "SELECT edit_params FROM images WHERE id = ?1",
        params![image_id],
        |row| row.get(0),
    )?;

    let params_json = current_params.unwrap_or_else(|| "{}".to_string());
    let id = uuid::Uuid::new_v4().to_string();
    db.conn.execute(
        "INSERT INTO snapshots (id, image_id, name, params_json) VALUES (?1, ?2, ?3, ?4)",
        params![id, image_id, name, params_json],
    )?;
    Ok(())
}

pub fn load_snapshot(
    db: &Database,
    snapshot_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let params_json: String = db.conn.query_row(
        "SELECT params_json FROM snapshots WHERE id = ?1",
        params![snapshot_id],
        |row| row.get(0),
    )?;
    Ok(params_json)
}

pub fn get_history(
    db: &Database,
    image_id: &str,
) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, image_id, action, params_json, created_at
         FROM edit_history WHERE image_id = ?1 ORDER BY created_at DESC LIMIT 100"
    )?;
    let rows = stmt.query_map(params![image_id], |row| {
        Ok(HistoryEntry {
            id: row.get(0)?,
            image_id: row.get(1)?,
            action: row.get(2)?,
            params_json: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}
