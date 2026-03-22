use chrono::Utc;
use log::debug;
use rusqlite::{Connection, OptionalExtension, Result};
use std::collections::HashSet;

use crate::repository::Id;

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub tags: HashSet<String>,
}

#[derive(Debug)]
pub struct NewActivity {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

impl Activity {
    pub fn create(conn: &mut Connection, new_activity: NewActivity) -> Result<Self> {
        let tx = conn.transaction()?;

        // Insert activity
        tx.execute(
            "INSERT INTO activities (name, description) VALUES (?1, ?2)",
            rusqlite::params![new_activity.name, new_activity.description],
        )?;
        let activity_id = tx.last_insert_rowid();

        // Insert or reuse tags, then link them
        for tag_name in new_activity.tags {
            // Upsert tag
            tx.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                rusqlite::params![tag_name],
            )?;

            let tag_id: i64 = tx.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                rusqlite::params![tag_name],
                |row| row.get(0),
            )?;

            // Link activity to tag
            tx.execute(
                "INSERT OR IGNORE INTO activities_tags (activity_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![activity_id, tag_id],
            )?;
        }

        tx.commit()?;
        Self::get_by_id(conn, activity_id)
    }

    pub fn get_by_id(conn: &Connection, id: Id) -> Result<Self> {
        let activity = conn.query_row(
            "SELECT id, name, description FROM activities WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(ActivityBase {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                })
            },
        )?;

        let mut tags = HashSet::new();
        let mut stmt = conn.prepare(
            "SELECT t.name FROM activities_tags at
             JOIN tags t ON at.tag_id = t.id
             WHERE at.activity_id = ?1",
        )?;
        let tag_rows = stmt.query_map(rusqlite::params![id], |row| row.get(0))?;
        for tag_row in tag_rows {
            tags.insert(tag_row?);
        }

        Ok(Activity {
            id: activity.id,
            name: activity.name,
            description: activity.description,
            tags,
        })
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt =
            conn.prepare("SELECT id, name, description FROM activities ORDER BY name")?;
        let activities = stmt
            .query_map([], |row| {
                Ok(ActivityBase {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                })
            })?
            .filter_map(Result::ok)
            .map(|base| Self::get_by_id(conn, base.id))
            .collect::<Result<Vec<_>>>()?;

        Ok(activities)
    }

    pub fn update(
        conn: &mut Connection,
        id: Id,
        name: Option<&str>,
        description: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<()> {
        let tx = conn.transaction()?;

        if let Some(name) = name {
            tx.execute(
                "UPDATE activities SET name = ?1 WHERE id = ?2",
                rusqlite::params![name, id],
            )?;
        }

        if let Some(description) = description {
            tx.execute(
                "UPDATE activities SET description = ?1 WHERE id = ?2",
                rusqlite::params![description, id],
            )?;
        }

        if let Some(new_tags) = tags {
            // Delete existing links
            tx.execute(
                "DELETE FROM activities_tags WHERE activity_id = ?1",
                rusqlite::params![id],
            )?;

            for tag_name in new_tags {
                tx.execute(
                    "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                    rusqlite::params![tag_name],
                )?;

                let tag_id: Id = tx.query_row(
                    "SELECT id FROM tags WHERE name = ?1",
                    rusqlite::params![tag_name],
                    |row| row.get(0),
                )?;

                tx.execute(
                    "INSERT INTO activities_tags (activity_id, tag_id) VALUES (?1, ?2)",
                    rusqlite::params![id, tag_id],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: Id) -> Result<()> {
        conn.execute(
            "DELETE FROM activities WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    pub fn get_current_ongoing(conn: &Connection) -> Result<Option<Self>> {
        // Get ongoing log's activity_id first
        let activity_id_opt: Option<Id> = conn
            .query_row(
                "SELECT activity_id FROM logs WHERE ends_at IS NULL LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match activity_id_opt {
            Some(activity_id) => {
                let activity = Self::get_by_id(conn, activity_id)?;

                let starts_at_str: Option<String> = conn
                    .query_row(
                        "SELECT starts_at FROM logs WHERE activity_id = ?1 AND ends_at IS NULL",
                        rusqlite::params![activity_id],
                        |row| row.get(0),
                    )
                    .optional()?;

                let ongoing_activity = activity;
                if let Some(start_str) = starts_at_str {
                    // Note: starts_at attached to Activity for convenience
                    // You could also return a separate OngoingActivity struct
                    debug!(
                        "Activity {} ongoing since {}",
                        ongoing_activity.name, start_str
                    );
                }

                Ok(Some(ongoing_activity))
            }
            None => Ok(None),
        }
    }

    pub fn start(conn: &mut Connection, activity_id: Id) -> Result<()> {
        let tx = conn.transaction()?;

        // End any existing ongoing log
        tx.execute(
            "UPDATE logs SET ends_at = CURRENT_TIMESTAMP WHERE ends_at IS NULL",
            [],
        )?;

        // Start new log for this activity
        let now = Utc::now();
        tx.execute(
            "INSERT INTO logs (activity_id, starts_at) VALUES (?1, ?2)",
            rusqlite::params![activity_id, now.to_rfc3339()],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn stop_current(conn: &Connection) -> Result<()> {
        conn.execute(
            "UPDATE logs SET ends_at = CURRENT_TIMESTAMP WHERE ends_at IS NULL",
            [],
        )?;
        Ok(())
    }
}

// Helper struct for base activity data
#[derive(Debug)]
struct ActivityBase {
    id: i64,
    name: String,
    description: Option<String>,
}
