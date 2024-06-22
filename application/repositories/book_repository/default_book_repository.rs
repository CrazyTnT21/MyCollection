use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tokio_postgres::Client;

use domain::entities::book::Book;
use domain::enums::language::Language;
use domain::items_total::ItemsTotal;
use domain::pagination::Pagination;
use from_row::Table;
use repositories::book_repository::BookRepository;
use repositories::image_repository::ImageRepository;

use crate::convert_to_sql::{to_i32};
use crate::enums::db_language::DbLanguage;
use crate::fallback_unwrap::{fallback_unwrap, fallback_unwrap_ref};
use crate::schemas::db_book::DbBook;
use crate::schemas::db_book_translation::DbBookTranslation;
use crate::schemas::db_franchise::DbFranchise;
use crate::schemas::db_image::DbImage;
use crate::select::combined_tuple::CombinedType;
use crate::select::conditions::column_equal::ColumnEqual;
use crate::select::conditions::column_null::ColumnNull;
use crate::select::conditions::value_ilike::ValueILike;
use crate::select::conditions::value_equal::ValueEqual;
use crate::select::conditions::value_in::ValueIn;
use crate::select::expression::Expression;
use crate::select::Select;

pub struct DefaultBookRepository<'a> {
  client: &'a Client,
  default_language: DbLanguage,
  image_repository: Arc<dyn ImageRepository + 'a>,
}

impl<'a> DefaultBookRepository<'a> {
  pub fn new(client: &'a Client, default_language: Language, image_repository: Arc<dyn ImageRepository + 'a>) -> DefaultBookRepository<'a> {
    DefaultBookRepository { client, default_language: default_language.into(), image_repository }
  }

  async fn books_from_tuple(&self, items: Vec<BookColumns>) -> Result<Vec<Book>, Box<dyn Error>> {
    if items.is_empty() {
      return Ok(vec![]);
    }

    let image_ids = image_ids(&items);
    let mut images = self.image_repository.get_by_ids(&image_ids).await?;

    items
      .into_iter()
      .map(|item| {
        let book_translation = fallback_unwrap(item.1, item.2);
        let franchise = item.5.map(|x| x.to_entity());
        let image_index = images.iter().position(|y| y.id == book_translation.fk_cover as u32).unwrap();
        let image = images.swap_remove(image_index);

        Ok(item.0.to_entity(book_translation, image, franchise))
      })
      .collect()
  }
  async fn book_from_tuple(&self, item: BookColumns) -> Result<Book, Box<dyn Error>> {
    let image_id = fallback_unwrap_ref(item.3.as_ref(), item.4.as_ref()).id as u32;
    let image = self.image_repository.get_by_id(image_id).await?.unwrap();
    let book_translation = fallback_unwrap(item.1, item.2);
    let franchise = item.5.map(|x| x.to_entity());
    Ok(item.0.to_entity(book_translation, image, franchise))
  }
}

fn image_ids(items: &[BookColumns]) -> Vec<u32> {
  items
    .iter()
    .map(|x| fallback_unwrap_ref(x.3.as_ref(), x.4.as_ref()).id as u32)
    .collect()
}

#[async_trait]
impl BookRepository for DefaultBookRepository<'_> {
  async fn get(&self, language: Language, pagination: Pagination) -> Result<ItemsTotal<Book>, Box<dyn Error>> {
    let language = DbLanguage::from(language);

    let total = Select::new::<DbBook>()
      .count()
      .transform(|x| book_joins(x, &language, &self.default_language))
      .get_single(self.client).await?
      .expect("Count should return one row");
    let total = total.0 as usize;

    let books = book_select(&language, &self.default_language)
      .pagination(pagination)
      .query(self.client)
      .await?;

    let books = self.books_from_tuple(books).await?;
    Ok(ItemsTotal { items: books, total })
  }

  async fn get_by_id(&self, id: u32, language: Language) -> Result<Option<Book>, Box<dyn Error>> {
    let id = id as i32;
    let language = DbLanguage::from(language);

    let select = book_select(&language, &self.default_language)
      .where_expression(Expression::new(ValueEqual::new(("book", "id"), id)));

    let Some(value) = select.get_single(self.client).await? else {
      return Ok(None);
    };
    Ok(Some(self.book_from_tuple(value).await?))
  }

  async fn get_by_title(&self, title: &str, language: Language, pagination: Pagination) -> Result<ItemsTotal<Book>, Box<dyn Error>> {
    let title = format!("%{title}%");
    let language = DbLanguage::from(language);

    let total = Select::new::<DbBook>()
      .count()
      .transform(|x| book_joins(x, &language, &self.default_language))
      .where_expression(Expression::new(ValueILike::new(("book_translation", "title"), &title))
        .or(Expression::new(ValueILike::new(("book_translation_fallback", "title"), &title))))
      .get_single(self.client).await?
      .expect("Count should return one row");

    let total = total.0 as usize;

    let books = book_select(&language, &self.default_language)
      .where_expression(Expression::new(ValueILike::new(("book_translation", "title"), &title))
        .or(Expression::new(ValueILike::new(("book_translation_fallback", "title"), &title))))
      .pagination(pagination)
      .query(self.client)
      .await?;
    let books = self.books_from_tuple(books).await?;
    Ok(ItemsTotal { items: books, total })
  }

  async fn get_by_ids(&self, ids: &[u32], language: Language) -> Result<Vec<Book>, Box<dyn Error>> {
    let language = DbLanguage::from(language);
    let ids = to_i32(ids);

    let books = book_select(&language, &self.default_language)
      .where_expression(Expression::new(ValueIn::new((DbBook::TABLE_NAME, "id"), &ids)))
      .query(self.client)
      .await?;

    let books = self.books_from_tuple(books).await?;

    Ok(books)
  }

  async fn filter_existing(&self, books: &[u32]) -> Result<Vec<u32>, Box<dyn Error>> {
    let books = to_i32(books);

    let count = Select::new::<DbBook>()
      .column::<i32>(DbBook::TABLE_NAME, "id")
      .where_expression(Expression::new(ValueIn::new((DbBook::TABLE_NAME, "id"), &books)))
      .query(self.client)
      .await?
      .into_iter()
      .map(|x| { x.0 as u32 })
      .collect();
    Ok(count)
  }
}

fn book_select<'a>(language: &'a DbLanguage, fallback_language: &'a DbLanguage) -> Select<'a, BookColumns> {
  book_select_columns()
    .transform(|x| book_joins(x, language, fallback_language))
}

fn book_joins<'a, T: from_row::FromRow<DbType=T> + CombinedType>(select: Select<'a, T>, language: &'a DbLanguage, fallback_language: &'a DbLanguage) -> Select<'a, T> {
  select.left_join::<DbBookTranslation>(Some("book_translation"),
                                        Expression::new(ColumnEqual::new(("book_translation", "fktranslation"), ("book", "id")))
                                          .and(Expression::new(ValueEqual::new(("book_translation", "language"), language))))
    .left_join::<DbBookTranslation>(Some("book_translation_fallback"),
                                    Expression::new(ColumnEqual::new(("book", "id"), ("book_translation_fallback", "fktranslation")))
                                      .and(Expression::new(ColumnNull::new(("book_translation", "fktranslation"))))
                                      .and(Expression::new(ValueEqual::new(("book_translation_fallback", "language"), fallback_language))))
    .left_join::<DbImage>(Some("cover"), Expression::new(ColumnEqual::new(("cover", "id"), ("book_translation", "fkcover"))))
    .left_join::<DbImage>(Some("cover_fallback"), Expression::new(ColumnEqual::new(("cover_fallback", "id"), ("book_translation_fallback", "fkcover"))))
    .left_join::<DbFranchise>(None, Expression::new(ColumnEqual::new(("book", "fkfranchise"), ("franchise", "id"))))
}

fn book_select_columns<'a>() -> Select<'a, BookColumns> {
  Select::new::<DbBook>()
    .columns::<DbBook>("book")
    .columns::<Option<DbBookTranslation>>("book_translation")
    .columns::<Option<DbBookTranslation>>("book_translation_fallback")
    .columns::<Option<DbImage>>("cover")
    .columns::<Option<DbImage>>("cover_fallback")
    .columns::<Option<DbFranchise>>("franchise")
}

type BookColumns = (DbBook, Option<DbBookTranslation>, Option<DbBookTranslation>, Option<DbImage>, Option<DbImage>, Option<DbFranchise>);
