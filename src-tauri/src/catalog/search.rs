use rusqlite::params;
use crate::catalog::db::Database;
use crate::catalog::models::ImageRecord;

pub fn search_images(
    db: &Database,
    query: &str,
    rating_min: Option<u8>,
    color_label: Option<&str>,
    flag: Option<&str>,
) -> Result<Vec<ImageRecord>, Box<dyn std::error::Error>> {
    let mut sql = String::from(
        "SELECT DISTINCT i.id, i.file_path, i.file_name, i.format, i.width, i.height,
                i.date_taken, i.rating, i.color_label, i.flag, i.camera, i.lens,
                i.iso, i.focal_length, i.aperture, i.shutter_speed, i.edit_params, i.created_at
         FROM images i
         LEFT JOIN image_tags it ON i.id = it.image_id
         LEFT JOIN tags t ON it.tag_id = t.id
         WHERE 1=1"
    );
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1;

    if !query.is_empty() {
        sql.push_str(&format!(" AND (i.file_name LIKE ?{p} OR i.camera LIKE ?{p} OR t.name LIKE ?{p})", p = idx));
        param_values.push(Box::new(format!("%{}%", query)));
        idx += 1;
    }
    if let Some(min_r) = rating_min {
        sql.push_str(&format!(" AND i.rating >= ?{}", idx));
        param_values.push(Box::new(min_r));
        idx += 1;
    }
    if let Some(cl) = color_label {
        sql.push_str(&format!(" AND i.color_label = ?{}", idx));
        param_values.push(Box::new(cl.to_string()));
        idx += 1;
    }
    if let Some(f) = flag {
        sql.push_str(&format!(" AND i.flag = ?{}", idx));
        param_values.push(Box::new(f.to_string()));
    }
    sql.push_str(" ORDER BY i.date_taken DESC LIMIT 500");

    let mut stmt = db.conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(refs.as_slice(), |row| {
        Ok(ImageRecord {
            id: row.get(0)?, file_path: row.get(1)?, file_name: row.get(2)?,
            format: row.get(3)?, width: row.get(4)?, height: row.get(5)?,
            date_taken: row.get(6)?, rating: row.get(7)?, color_label: row.get(8)?,
            flag: row.get(9)?, camera: row.get(10)?, lens: row.get(11)?,
            iso: row.get(12)?, focal_length: row.get(13)?, aperture: row.get(14)?,
            shutter_speed: row.get(15)?, edit_params: row.get(16)?,
            tags: Vec::new(), created_at: row.get(17)?,
        })
    })?;
    let mut images = Vec::new();
    for row in rows { images.push(row?); }
    Ok(images)
}
