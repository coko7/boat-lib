use anyhow::{Result, anyhow};
use log::debug;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;

pub fn deserialize_record<T>(input: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(input.as_bytes());
    let mut iter = rdr.deserialize();

    let record: T = iter.next().ok_or_else(|| anyhow!("no record found"))??;
    Ok(record)
}

pub fn deserialize<T>(input: &str) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(input.as_bytes());
    let mut out = Vec::new();

    for result in rdr.deserialize() {
        let record: T = result?;
        debug!("{record:?}");
        out.push(record);
    }

    Ok(out)
}

pub fn serialize<T: Serialize>(data: &[T]) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(vec![]);

    for record in data {
        wtr.serialize(record)?;
    }
    wtr.flush()?;

    let bytes = wtr.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

pub fn deserialize_hashset<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(HashSet::new());
    }

    Ok(s.split('|').map(|v| v.trim().to_string()).collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        models::{Activity, TrackingEntry},
        repository::RepositoryItem,
        utils,
    };

    use super::*;

    #[test]
    fn test_parse_tracking_csv() -> Result<()> {
        utils::init_test_logger();
        assert_eq!(
            vec![TrackingEntry {
                id: "27e28d2ac61f35f7".to_string(),
                activity_id: "e6468a7c866eb71c".to_string(),
                start: utils::parse_local_dt("2026-03-16 08:00:00")?,
                end: Some(utils::parse_local_dt("2026-03-16 09:30:00")?),
            }],
            deserialize::<TrackingEntry>(
                "id;activity_id;start;end
27e28d2ac61f35f7;e6468a7c866eb71c;2026-03-16T08:00:00+01:00;2026-03-16T09:30:00+01:00"
            )?
        );
        Ok(())
    }

    #[test]
    fn test_parse_activities_csv() -> Result<()> {
        utils::init_test_logger();
        let mut tags = HashSet::new();
        tags.insert("code".to_string());
        tags.insert("cli".to_string());

        assert_eq!(
            vec![
                Activity::new("take out trash").set_id_fluent("1e702a61b5c30021".to_string()),
                Activity::new("cook pasta").set_id_fluent("2588c8f8ac72af67".to_string()),
                Activity::new("work on csv parser")
                    .set_id_fluent("bc7272ea7045d136".to_string())
                    .set_tags(tags)
            ],
            // TODO: handle duplicate activity IDss
            deserialize::<Activity>(
                "id;name;project;description;tags
1e702a61b5c30021;take out trash;;;
2588c8f8ac72af67;cook pasta;;;
bc7272ea7045d136;work on csv parser;;;cli|code"
            )?
        );
        Ok(())
    }
}
