use async_trait::async_trait;

use domain::entities::genre::Genre;
use domain::enums::language::Language;
use domain::items_total::ItemsTotal;
use domain::pagination::Pagination;
use repositories::genre_repository::GenreRepository;
use services::genre_service::GenreService;
use services::traits::service_error::ServiceError;

use crate::services::map_server_error;

pub struct DefaultGenreService<'a> {
  genre_repository: &'a dyn GenreRepository,
}

impl<'a> DefaultGenreService<'a> {
  pub fn new(genre_repository: &'a dyn GenreRepository) -> DefaultGenreService {
    DefaultGenreService { genre_repository }
  }
}

#[async_trait]
impl<'a> GenreService for DefaultGenreService<'a> {
  async fn get(&self, language: Language, pagination: Pagination) -> Result<ItemsTotal<Genre>, ServiceError> {
    self.genre_repository.get(language, pagination).await.map_err(map_server_error)
  }

  async fn get_by_id(&self, id: u32, language: Language) -> Result<Option<Genre>, ServiceError> {
    self.genre_repository.get_by_id(id, language).await.map_err(map_server_error)
  }

  async fn get_by_ids(&self, ids: &[i32], language: Language) -> Result<Vec<Genre>, ServiceError> {
    self.genre_repository.get_by_ids(ids, language).await.map_err(map_server_error)
  }

  async fn get_by_name(&self, name: &str, language: Language, pagination: Pagination) -> Result<ItemsTotal<Genre>, ServiceError> {
    self.genre_repository.get_by_name(name, language, pagination).await.map_err(map_server_error)
  }
}