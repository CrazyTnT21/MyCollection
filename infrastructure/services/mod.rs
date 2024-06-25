use std::error::Error;
use services::traits::service_error::ServiceError;

pub mod default_genre_service;
pub mod default_theme_service;
pub mod default_person_service;
pub mod default_character_service;
pub mod default_role_service;
pub mod file_service;
pub mod image_service;
pub mod user_service;
pub mod account_service;
pub mod book_service;
pub mod default_franchise_service;

fn map_server_error(error: Box<dyn Error>) -> ServiceError {
  ServiceError::ServerError(error)
}
