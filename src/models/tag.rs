use rusqlite::{Connection, OptionalExtension, Result};

use crate::repository::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: Id,
    pub name: String,
}

#[derive(Debug)]
pub struct NewTag {
    pub name: String,
}

impl Tag {
    pub fn create(conn: &Connection, new_tag: NewTag) -> Result<Self> {
        conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            rusqlite::params![new_tag.name],
        )?;

        let id: Id = conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            rusqlite::params![new_tag.name],
            |row| row.get(0),
        )?;

        Self::get_by_id(conn, id)
    }

    pub fn get_by_id(conn: &Connection, id: Id) -> Result<Self> {
        conn.query_row(
            "SELECT id, name FROM tags WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Self {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
    }

    pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<Self>> {
        conn.query_row(
            "SELECT id, name FROM tags WHERE name = ?1",
            rusqlite::params![name],
            |row| {
                Ok(Self {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
        .optional()
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT id, name FROM tags ORDER BY name")?;
        let tags = stmt
            .query_map([], |row| {
                Ok(Self {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(tags)
    }

    pub fn usage_stats(conn: &Connection) -> Result<Vec<(Self, Id)>> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, COUNT(DISTINCT at.activity_id) as activity_count
             FROM tags t 
             LEFT JOIN activities_tags at ON t.id = at.tag_id
             GROUP BY t.id 
             ORDER BY activity_count DESC, t.name",
        )?;

        let stats = stmt
            .query_map([], |row| {
                Ok((
                    Self {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    },
                    row.get(2)?,
                ))
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(stats)
    }

    pub fn delete(conn: &Connection, id: Id) -> Result<()> {
        conn.execute("DELETE FROM tags WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn delete_by_name(conn: &Connection, name: &str) -> Result<()> {
        conn.execute("DELETE FROM tags WHERE name = ?1", rusqlite::params![name])?;
        Ok(())
    }
}
