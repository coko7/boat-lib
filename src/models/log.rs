use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result};

use crate::repository::Id;

#[derive(Debug, Clone)]
pub struct Log {
    pub id: Id,
    pub activity_id: Id,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct NewLog {
    pub activity_id: Id,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct LogWithActivity {
    pub id: i64,
    pub activity_id: i64,
    pub activity_name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}

impl Log {
    pub fn create(conn: &Connection, new_log: NewLog) -> Result<Self> {
        conn.execute(
            "INSERT INTO logs (activity_id, starts_at, ends_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![
                new_log.activity_id,
                new_log.starts_at.to_rfc3339(),
                new_log.ends_at.map(|t| t.to_rfc3339())
            ],
        )?;

        let id = conn.last_insert_rowid();
        Self::get_by_id(conn, id)
    }

    pub fn get_by_id(conn: &Connection, id: Id) -> Result<Self> {
        conn.query_row(
            "SELECT id, activity_id, starts_at, ends_at FROM logs WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                let starts_at_str = row
                    .get::<_, Option<String>>(2)?
                    .expect("Database corruption: missing starts_at");
                let ends_at_str = row.get::<_, Option<String>>(3)?;

                Ok(Self {
                    id: row.get(0)?,
                    activity_id: row.get(1)?,
                    starts_at: parse_datetime(Some(starts_at_str))
                        .expect("Invalid starts_at datetime")
                        .unwrap(),
                    ends_at: parse_datetime(ends_at_str)?,
                })
            },
        )
    }

    pub fn get_for_activity(conn: &Connection, activity_id: Id) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, activity_id, starts_at, ends_at 
             FROM logs WHERE activity_id = ?1 ORDER BY starts_at DESC",
        )?;

        let logs = stmt
            .query_map(rusqlite::params![activity_id], |row| {
                let starts_at_str = row
                    .get::<_, Option<String>>(2)?
                    .expect("Database corruption: missing starts_at");
                let ends_at_str = row.get::<_, Option<String>>(3)?;

                Ok(Self {
                    id: row.get(0)?,
                    activity_id: row.get(1)?,
                    starts_at: parse_datetime(Some(starts_at_str))
                        .expect("Invalid starts_at datetime in database")
                        .unwrap(),
                    ends_at: parse_datetime(ends_at_str)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();

        Ok(logs)
    }

    pub fn recent_with_activities(conn: &Connection, limit: i64) -> Result<Vec<LogWithActivity>> {
        let mut stmt = conn.prepare(
            "SELECT l.id, l.activity_id, a.name, l.starts_at, l.ends_at 
             FROM logs l 
             JOIN activities a ON l.activity_id = a.id 
             ORDER BY l.starts_at DESC LIMIT ?1",
        )?;

        let logs = stmt
            .query_map(rusqlite::params![limit], |row| {
                let activity_name = row
                    .get::<_, Option<String>>(2)?
                    .expect("Database corruption: missing activity name");
                let starts_at_str = row
                    .get::<_, Option<String>>(3)?
                    .expect("Database corruption: missing starts_at");
                let ends_at_str = row.get::<_, Option<String>>(4)?;

                Ok(LogWithActivity {
                    id: row.get(0)?,
                    activity_id: row.get(1)?,
                    activity_name,
                    starts_at: parse_datetime(Some(starts_at_str))
                        .expect("Invalid starts_at datetime in database")
                        .unwrap(),
                    ends_at: parse_datetime(ends_at_str)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();

        Ok(logs)
    }

    pub fn update_end(conn: &Connection, id: Id, ends_at: DateTime<Utc>) -> Result<()> {
        conn.execute(
            "UPDATE logs SET ends_at = ?1 WHERE id = ?2",
            rusqlite::params![ends_at.to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: Id) -> Result<()> {
        conn.execute("DELETE FROM logs WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}

fn parse_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    match value {
        Some(s) => Ok(Some(
            DateTime::parse_from_rfc3339(&s)
                .map_err(|_| rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc),
        )),
        None => Ok(None),
    }
}
