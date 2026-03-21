use anyhow::Result;
use std::fmt::Debug;

pub mod file_repository;

pub type Id = String;

pub trait Repository<T: RepositoryItem>: Debug {
    fn create(&mut self, item: T) -> Result<Id>;
    fn get(&self, id: Id) -> Result<Option<T>>;
    fn list(&self) -> Result<Vec<T>>;
    fn update(&mut self, item: T) -> Result<()>;
    fn delete(&mut self, id: Id) -> Result<()>;
}

pub trait RepositoryItem {
    fn id(&self) -> Id;
    fn set_id(&mut self, value: Id);
}
