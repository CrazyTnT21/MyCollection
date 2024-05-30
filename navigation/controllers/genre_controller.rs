use axum::{Json, Router};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use bb8_postgres::bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

use services::genre_service::GenreService;

use crate::controllers::{append_content_language_header, content_language_header, convert_service_error, DEFAULT_LANGUAGE, get_language, set_pagination_limit};
use crate::database_connection::DatabaseConnection;
use crate::extractors::headers::accept_language::AcceptLanguageHeader;
use crate::extractors::query_pagination::QueryPagination;
use crate::implementations::{get_genre_repository, get_genre_service};
use crate::openapi::params::header::accept_language::AcceptLanguageParam;
use crate::openapi::params::path::id::IdParam;
use crate::openapi::params::path::name::NameParam;
use crate::openapi::params::query::count::CountParam;
use crate::openapi::params::query::page::PageParam;
use crate::openapi::responses::bad_request::BadRequest;
use crate::openapi::responses::not_found::NotFound;
use crate::openapi::responses::server_error::ServerError;

pub mod genre_doc;

pub fn routes(pool: Pool<PostgresConnectionManager<NoTls>>) -> Router {
  Router::new()
    .route("/", get(get_items))
    .route("/:id", get(get_by_id))
    .route("/name/:name", get(get_by_name))
    .with_state(pool)
}

#[utoipa::path(get, path = "",
responses(
(status = 200, description = "Returned genres", body = GenresTotal), ServerError, BadRequest),
params(AcceptLanguageParam, PageParam, CountParam),
tag = "Genres"
)]
async fn get_items(AcceptLanguageHeader(languages): AcceptLanguageHeader, connection: DatabaseConnection, Query(mut pagination): Query<QueryPagination>) -> impl IntoResponse {
  let connection = connection.0;
  let repository = get_genre_repository(&connection,DEFAULT_LANGUAGE);
  let service = get_genre_service(&repository);

  let language = get_language(languages, DEFAULT_LANGUAGE);
  set_pagination_limit(&mut pagination);

  println!("Route for genres in {}", language);

  let mut content_language = content_language_header(language);
  append_content_language_header(&mut content_language, DEFAULT_LANGUAGE);

  match service.get(language, pagination.into()).await {
    Ok(genres) => Ok((StatusCode::OK, content_language, Json(genres))),
    Err(error) => Err(convert_service_error(error))
  }
}

#[utoipa::path(get, path = "/{id}",
responses(
(status = 200, description = "Returned genre based on the id", body = Genre), ServerError, BadRequest, NotFound),
params(IdParam, AcceptLanguageParam),
tag = "Genres"
)]
async fn get_by_id(Path(id): Path<u32>, AcceptLanguageHeader(languages): AcceptLanguageHeader, connection: DatabaseConnection) -> impl IntoResponse {
  let connection = connection.0;
  let repository = get_genre_repository(&connection, DEFAULT_LANGUAGE);
  let service = get_genre_service(&repository);

  let language = get_language(languages, DEFAULT_LANGUAGE);

  println!("Route for a genre with id {} in {}", id, language);

  let mut content_language = content_language_header(language);
  append_content_language_header(&mut content_language, DEFAULT_LANGUAGE);

  match service.get_by_id(id, language).await {
    Ok(item) => Ok((StatusCode::OK, content_language, Json(item))),
    Err(error) => Err(convert_service_error(error))
  }
}


#[utoipa::path(get, path = "/name/{name}",
responses(
(status = 200, description = "Returned genres based on the name", body = GenresTotal), ServerError, BadRequest),
params(NameParam, AcceptLanguageParam, PageParam, CountParam),
tag = "Genres"
)]
async fn get_by_name(Path(name): Path<String>, AcceptLanguageHeader(languages): AcceptLanguageHeader, connection: DatabaseConnection, Query(mut pagination): Query<QueryPagination>) -> impl IntoResponse {
  let connection = connection.0;
  let repository = get_genre_repository(&connection, DEFAULT_LANGUAGE);
  let service = get_genre_service(&repository);

  let language = get_language(languages, DEFAULT_LANGUAGE);
  set_pagination_limit(&mut pagination);

  println!("Route for genres with the name {} in {}", name, language);

  let mut content_language = content_language_header(language);
  append_content_language_header(&mut content_language, DEFAULT_LANGUAGE);

  match service.get_by_name(&name, language, pagination.into()).await {
    Ok(items) => Ok((StatusCode::OK, content_language, Json(items))),
    Err(error) => Err(convert_service_error(error))
  }
}