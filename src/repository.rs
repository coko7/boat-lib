use anyhow::Result;
use std::fmt::Debug;

pub type Id = i64;

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
