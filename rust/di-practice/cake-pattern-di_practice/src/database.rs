use anyhow::Result;
use common::User;

pub trait Database: Send + Sync + 'static {}
// pub struct MySql {}
// impl Database for MySql {}
// pub struct MySqlTest {}
// impl Database for MySqlTest {}

pub trait UsesDatabase: Send + Sync + 'static {
    fn find_user(&self, id: String) -> Result<Option<User>>;
}

impl<T: Database> UsesDatabase for T {
    fn find_user(&self, id: String) -> Result<Option<User>> {
        Ok(Some(User {
            id: "id-a".to_string(),
            effective: true,
        }))
    }
}

pub trait ProvidesDatabase: Send + Sync + 'static {
    type T: UsesDatabase;
    fn database(&self) -> &Self::T;
}
