use anyhow::{Result, ensure};
use log::debug;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    csv_io,
    repository::{Id, Repository, RepositoryItem},
    utils,
};

#[derive(Debug)]
pub struct CsvFileRepository<T>
where
    T: Eq + std::hash::Hash,
{
    path: PathBuf,
    items: HashMap<Id, T>,
}

impl<T> CsvFileRepository<T>
where
    T: RepositoryItem
        + Clone
        + std::fmt::Debug
        + Eq
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>,
{
    pub fn new(path: &Path) -> Result<Self> {
        let mut repo = CsvFileRepository {
            path: path.to_path_buf(),
            items: HashMap::new(),
        };
        repo.initialize()?;
        repo.load()?;
        Ok(repo)
    }

    fn initialize(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        match File::create_new(&self.path) {
            Ok(mut file) => {
                file.flush()?;
                debug!("created new file repository: {}", self.path.display());
            }
            Err(_) => debug!("file repository already exists: {}", self.path.display()),
        }

        Ok(())
    }

    fn load(&mut self) -> Result<()> {
        let raw_csv = fs::read_to_string(&self.path)?;
        debug!("loaded raw csv: {raw_csv}");

        let items = csv_io::deserialize::<T>(&raw_csv)?;
        debug!("loaded items: {items:?}");

        self.items = items.into_iter().map(|item| (item.id(), item)).collect();
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        let items: Vec<_> = self.items.values().collect();
        let csv_str = csv_io::serialize(&items)?;
        debug!("converted data to csv: {items:?}");

        file.write_all(csv_str.as_bytes())?;
        debug!("wrote csv to file: {}", self.path.display());
        Ok(())
    }
}

impl<T> Repository<T> for CsvFileRepository<T>
where
    T: RepositoryItem
        + Clone
        + std::fmt::Debug
        + Eq
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>,
{
    fn create(&mut self, item: T) -> Result<Id> {
        ensure!(
            item.id().is_empty(),
            "the ID of a new item should not be set!"
        );

        let mut clone = item.clone();
        let id = utils::generate_rand_hash();
        clone.set_id(id.clone());
        self.items.insert(id.clone(), clone);
        self.save()?;
        Ok(id)
    }

    fn get(&self, id: Id) -> Result<Option<T>> {
        Ok(self.items.get(&id).cloned())
    }

    fn list(&self) -> Result<Vec<T>> {
        Ok(self.items.values().cloned().collect())
    }

    fn update(&mut self, item: T) -> Result<()> {
        ensure!(
            !item.id().is_empty(),
            "the ID of an existing item should be set!"
        );

        let id = item.id();
        self.items.insert(id, item);
        self.save()
    }

    fn delete(&mut self, id: Id) -> Result<()> {
        self.items.remove(&id);
        self.save()
    }
}
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     use crate::repository::Repository;
//     use serde::{Deserialize, Serialize};
//     use std::fs;
//
//     #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
//     struct Dummy {
//         id: u32,
//         name: String,
//     }
//
//     fn example_entries() -> Vec<Dummy> {
//         vec![
//             Dummy {
//                 id: 1,
//                 name: "A".into(),
//             },
//             Dummy {
//                 id: 2,
//                 name: "B".into(),
//             },
//         ]
//     }
//
//     #[test]
//     fn test_initialize_creates_empty_file() -> Result<()> {
//         let dir = tempfile::tempdir()?;
//         let file_path = dir.path().join("repo.csv");
//
//         let repo = FileRepository::new(&file_path);
//         <FileRepository<Dummy> as Repository<Dummy>>::initialize(&repo)?;
//
//         let content = fs::read_to_string(&file_path)?;
//         assert!(content.trim().is_empty());
//         Ok(())
//     }
//
//     #[test]
//     fn test_save_and_load_roundtrip() -> Result<()> {
//         let dir = tempfile::tempdir()?;
//         let file_path = dir.path().join("repo.csv");
//
//         let repo = FileRepository::new(&file_path);
//         <FileRepository<Dummy> as Repository<Dummy>>::initialize(&repo)?;
//
//         let entries = example_entries();
//         repo.save_all(&entries)?;
//
//         let loaded = repo.load_all()?;
//         assert_eq!(entries, loaded);
//         Ok(())
//     }
//
//     #[test]
//     fn test_load_all_empty_returns_empty_vec() -> Result<()> {
//         let dir = tempfile::tempdir()?;
//         let file_path = dir.path().join("repo.csv");
//
//         let repo = FileRepository::new(&file_path);
//         <FileRepository as Repository<Dummy>>::initialize(&repo)?;
//
//         let loaded: Vec<Dummy> = repo.load_all()?;
//         assert!(loaded.is_empty());
//         Ok(())
//     }
// }
