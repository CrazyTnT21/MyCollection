use std::sync::Arc;

use async_trait::async_trait;

use domain::entities::book::book_statistic::BookStatistic;
use domain::entities::book::Book;
use domain::enums::language::Language;
use domain::items_total::ItemsTotal;
use domain::pagination::Pagination;
use domain::slug::Slug;
use repositories::book_repository::BookRepository;
use services::book_service::{BookService, BookServiceError};
use services::traits::service_error::ServiceError;
use services::traits::service_error::ServiceError::ClientError;

pub struct DefaultBookService<'a> {
  book_repository: Arc<dyn BookRepository + 'a>,
}

impl<'a> DefaultBookService<'a> {
  pub fn new(book_repository: Arc<dyn BookRepository + 'a>) -> DefaultBookService<'a> {
    DefaultBookService { book_repository }
  }
}

#[async_trait]
impl BookService for DefaultBookService<'_> {
  async fn get(
    &self,
    language: Language,
    pagination: Pagination,
  ) -> Result<ItemsTotal<Book>, ServiceError<BookServiceError>> {
    Ok(self.book_repository.get(language, pagination).await?)
  }

  async fn get_by_id(&self, id: u32, language: Language) -> Result<Option<Book>, ServiceError<BookServiceError>> {
    Ok(self.book_repository.get_by_id(id, language).await?)
  }

  async fn get_by_title(
    &self,
    title: &str,
    language: Language,
    pagination: Pagination,
  ) -> Result<ItemsTotal<Book>, ServiceError<BookServiceError>> {
    Ok(self.book_repository.get_by_title(title, language, pagination).await?)
  }
  async fn get_by_slug(&self, slug: &Slug, language: Language) -> Result<Option<Book>, ServiceError<BookServiceError>> {
    Ok(self.book_repository.get_by_slug(slug, language).await?)
  }

  async fn get_statistics(&self, book_ids: &[u32]) -> Result<Vec<BookStatistic>, ServiceError<BookServiceError>> {
    let existing = self.book_repository.filter_existing(book_ids).await?;
    if existing.len() != book_ids.len() {
      let non_existent_books = filter_non_existent(book_ids, &existing);
      return Err(ClientError(BookServiceError::NonExistentBooks(non_existent_books)));
    };
    Ok(self.book_repository.get_statistics(book_ids).await?)
  }
}
fn filter_non_existent(items: &[u32], existing: &[u32]) -> Vec<u32> {
  items
    .iter()
    .filter_map(|x| existing.iter().find(|y| **y == *x).map_or(Some(*x), |_| None))
    .collect()
}
