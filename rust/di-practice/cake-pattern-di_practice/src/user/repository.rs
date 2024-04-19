use anyhow::Result;
use common::User;

use crate::database::ProvidesDatabase;
use crate::database::UsesDatabase;

pub trait UsesUserRepository: Send + Sync + 'static {
    fn find_user(&self, id: String) -> Result<Option<User>>;
}

pub trait UserRepository: ProvidesDatabase {}

impl<T: UserRepository> UsesUserRepository for T {
    fn find_user(&self, id: String) -> Result<Option<User>> {
        self.database().find_user(id)
    }
}

pub trait ProvidesUserRepository: Send + Sync + 'static {
    type T: UsesUserRepository;
    fn user_repository(&self) -> &Self::T;
}
