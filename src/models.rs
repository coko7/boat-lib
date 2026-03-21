use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::repository::{Id, RepositoryItem};

pub type Actime = DateTime<Local>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Activity {
    id: Id,
    name: String,
    project: Option<String>,
    description: Option<String>,
    #[serde(deserialize_with = "crate::csv_io::deserialize_hashset")]
    tags: HashSet<String>,
}

impl RepositoryItem for Activity {
    fn id(&self) -> Id {
        self.id.clone()
    }

    fn set_id(&mut self, value: Id) {
        self.id = value;
    }
}

impl Activity {
    pub fn new(name: &str) -> Activity {
        Activity {
            id: String::new(),
            name: name.to_string(),
            project: None,
            description: None,
            tags: HashSet::new(),
        }
    }

    pub fn set_id_fluent(mut self, value: Id) -> Self {
        self.set_id(value);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn tags(&self) -> &HashSet<String> {
        &self.tags
    }

    pub fn set_tags(mut self, tags: HashSet<String>) -> Self {
        self.tags = tags;
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TrackingEntry {
    pub id: Id,
    pub activity_id: Id,
    pub start: Actime,
    pub end: Option<Actime>,
}

impl RepositoryItem for TrackingEntry {
    fn id(&self) -> Id {
        self.id.clone()
    }

    fn set_id(&mut self, value: Id) {
        self.id = value;
    }
}

impl TrackingEntry {
    pub fn new(activity: Id, start: Actime) -> Self {
        Self {
            id: String::new(),
            activity_id: activity,
            start,
            end: None,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::{Local, TimeZone};
//     use std::collections::HashSet;
//
//     #[test]
//     fn test_new_activity_and_getters_setters() {
//         let mut act = Activity::new("abc", "run");
//         assert_eq!(act.id(), "abc");
//         assert_eq!(act.parent_id(), None);
//         assert_eq!(act.name(), "run");
//
//         act.set_name("walk");
//         assert_eq!(act.name(), "walk");
//
//         act.set_parent("parent1".to_string());
//         assert_eq!(act.parent_id(), Some("parent1".to_string()));
//
//         act.unset_parent();
//         assert_eq!(act.parent_id(), None);
//
//         let mut tags = HashSet::new();
//         tags.insert("urgent".to_string());
//         act.set_tags(tags.clone());
//         assert_eq!(act.tags(), &tags);
//     }
//
//     #[test]
//     fn test_from_definition() {
//         let mut tags = HashSet::new();
//         tags.insert("foo".to_string());
//
//         let def = ActivityDefinition {
//             id: "id1".to_string(),
//             parent_id: Some("pid".to_string()),
//             name: "Activity1".to_string(),
//             tags: tags.clone(),
//         };
//
//         let act = Activity::from_definition(def.clone());
//
//         assert_eq!(act.id(), &def.id);
//         assert_eq!(act.parent_id(), def.parent_id);
//         assert_eq!(act.name(), &def.name);
//         assert_eq!(act.tags(), &tags);
//         assert!(act.tracking().is_empty());
//     }
//
//     #[test]
//     fn test_register_log_and_tracking() {
//         let mut act = Activity::new("logid", "move");
//         let start = Local.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
//         let end = Some(Local.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap());
//         let log = TrackingEntry {
//             id: "logid".to_string(),
//             start,
//             end,
//         };
//
//         act.register_log(log.clone());
//         assert!(act.tracking().contains(&(log.start, log.end)));
//     }
// }
