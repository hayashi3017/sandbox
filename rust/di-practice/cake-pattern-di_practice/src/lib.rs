use database::{Database, ProvidesDatabase};
use user::{
    repository::{ProvidesUserRepository, UserRepository},
    service::{ProvidesUserService, UserService},
};

pub mod database;
pub mod user;

pub struct AppModule;
impl AppModule {
    pub fn new() -> Self {
        Self
    }
}

impl Database for AppModule {}
impl UserRepository for AppModule {}
impl UserService for AppModule {}
impl ProvidesDatabase for AppModule {
    type T = Self;
    fn database(&self) -> &Self::T {
        self
    }
}
impl ProvidesUserRepository for AppModule {
    type T = Self;
    fn user_repository(&self) -> &Self::T {
        self
    }
}
impl ProvidesUserService for AppModule {
    type T = Self;
    fn user_service(&self) -> &Self::T {
        self
    }
}

pub mod router {
    use actix_web::{get, web::Data, web::Path, HttpResponse};

    use crate::user::service::UsesUserService;
    use crate::ProvidesUserService;

    #[get("/users/{id}")]
    pub async fn find_user(id: Path<String>, app_module: Data<crate::AppModule>) -> HttpResponse {
        let user = app_module.user_service().find_user(id.into_inner());
        match user {
            Ok(Some(user)) => HttpResponse::Ok().json(user),
            Ok(None) => HttpResponse::NotFound().finish(),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }
}
