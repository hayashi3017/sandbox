use anyhow::Result;
use common::User;

use super::repository::{ProvidesUserRepository, UsesUserRepository};

pub trait UserService: ProvidesUserRepository {}

pub trait UsesUserService {
    fn find_user(&self, id: String) -> Result<Option<User>>;
}

impl<T: UserService> UsesUserService for T {
    fn find_user(&self, id: String) -> Result<Option<User>> {
        self.user_repository().find_user(id)
    }
}

pub trait ProvidesUserService: Send + Sync + 'static {
    type T: UsesUserService;
    fn user_service(&self) -> &Self::T;
}
