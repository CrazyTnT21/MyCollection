use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use tokio_postgres::{Client, Transaction};
use tokio_postgres::types::ToSql;

use from_row::Table;


pub struct Insert<'a> {
  into: &'a str,
  columns: &'a [&'a str],
  values: Vec<&'a [&'a (dyn ToSql + Sync)]>,
}

impl<'a> Insert<'a> {
  pub fn new<T: Table>(columns: &'a [&'a str]) -> Insert<'a> {
    Self::new_raw(T::TABLE_NAME, columns)
  }
  pub fn new_raw(into: &'a str, columns: &'a [&'a str]) -> Insert<'a> {
    Insert {
      into,
      columns,
      values: vec![],
    }
  }
  pub fn push(mut self, values: &'a [&'a (dyn ToSql + Sync)]) -> Self {
    self.values.push(values);
    self
  }

  pub async fn execute(&self, connection: &'a Client) -> Result<u64, InsertError> {
    self.invalid_length()?;

    connection.execute(&self.sql(), &self.values()).await.map_err(|x| InsertError::PostgresError(x))
  }

  pub async fn execute_transaction(&self, transaction: &'a Transaction<'a>) -> Result<u64, InsertError> {
    self.invalid_length()?;

    transaction.execute(&self.sql(), &self.values()).await.map_err(|x| InsertError::PostgresError(x))
  }

  pub async fn returning(&self, connection: &'a Client) -> Result<i32, InsertError> {
    if self.values.len() > 1 {
      return Err(InsertError::ReturningMoreThanOne.into());
    }
    self.invalid_length()?;

    let result = connection.query_one(&self.returning_sql(), &self.values()).await.map_err(|x| InsertError::PostgresError(x))?;

    Ok(result.get(0))
  }
  fn invalid_length(&self) -> Result<(), InsertError> {
    let columns_length = self.columns.len();
    for x in &self.values {
      let values_length = x.len();
      if values_length != columns_length {
        return Err(InsertError::InvalidLength(values_length, columns_length));
      }
    }
    Ok(())
  }
  pub async fn returning_transaction(&self, transaction: &'a Transaction<'a>) -> Result<i32, InsertError> {
    if self.values.len() > 1 {
      return Err(InsertError::ReturningMoreThanOne.into());
    }
    self.invalid_length()?;
    let result = transaction.query_one(&self.returning_sql(), &self.values()).await.map_err(|x| InsertError::PostgresError(x))?;
    Ok(result.get(0))
  }

  pub fn sql(&self) -> String {
    let into = self.into;
    if self.values.is_empty() {
      return format!("INSERT INTO {into} DEFAULT VALUES;");
    }

    let columns = self.columns_sql();
    let values = self.values_sql();
    format!(r"INSERT INTO {into}({columns}) values {values}")
  }

  pub fn returning_sql(&self) -> String {
    let into = self.into;
    if self.values.is_empty() {
      return format!("INSERT INTO {into} DEFAULT VALUES RETURNING id;");
    }

    let columns = self.columns_sql();
    let values = self.values_sql();
    format!(r"INSERT INTO {into}({columns}) values {values} returning id;")
  }

  fn columns_sql(&self) -> String {
    self.columns.join(",")
  }

  fn values_sql(&self) -> String {
    let mut total = 0;
    self.values.iter().map(|x| {
      let result = (1..x.len() + 1)
        .collect::<Vec<usize>>()
        .iter()
        .map(|i| {
          total += 1;
          format!("${}", total)
        })
        .collect::<Vec<String>>()
        .join(",");
      format!("({result})")
    }).collect::<Vec<String>>().join(",")
  }

  fn values(&self) -> Vec<&'a (dyn ToSql + Sync)> {
    let mut result = vec![];
    self.values
      .iter().for_each(|x| {
      x.iter().for_each(|x| {
        result.push(*x)
      })
    });
    result
  }
}

#[derive(Debug)]
pub enum InsertError {
  ReturningMoreThanOne,
  InvalidLength(usize, usize),
  PostgresError(tokio_postgres::Error),
}


impl Display for InsertError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      InsertError::ReturningMoreThanOne => write!(f, "Returning is not supported for more than 1 value at a time"),
      InsertError::PostgresError(value) => std::fmt::Display::fmt(&value, f),
      InsertError::InvalidLength(a, b) => write!(f, "values length '{a}' does not match the length of the columns '{b}'")
    }
  }
}

impl Error for InsertError {}
