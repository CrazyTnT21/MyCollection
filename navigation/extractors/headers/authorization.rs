use std::str::FromStr;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum JWTError {
  AuthorizationMissing,
  MissingJWT,
}

#[derive(Debug)]
pub struct JWTAuthorization {
  pub token: String,
}

impl FromStr for JWTAuthorization {
  type Err = JWTError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut split_value = s.trim().split("bearer: ");
    let (_, token) = (split_value.next(), split_value.next());
    let token = token.ok_or(JWTError::MissingJWT)?.to_string();
    Ok(JWTAuthorization { token })
  }
}

#[async_trait]
impl<S> FromRequestParts<S> for JWTAuthorization
where
  S: Send + Sync,
{
  type Rejection = JWTError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let x = parts
      .headers
      .get("authorization")
      .ok_or(JWTError::AuthorizationMissing)?
      .to_str()
      .unwrap();

    JWTAuthorization::from_str(x)
  }
}

impl IntoResponse for JWTError {
  fn into_response(self) -> Response {
    let message = match self {
      JWTError::AuthorizationMissing => "Authorization header missing",
      JWTError::MissingJWT => "Invalid bearer prefix or jwt missing",
    };
    Response::new(message.into())
  }
}
