use tokio_postgres::Row;
use from_row::FromRow;

#[derive(FromRow, Debug)]
#[rename = "bookinvolved"]
pub struct DbBookInvolved {
  #[rename = "fkbook"]
  pub fk_book: i32,
  #[rename = "fkrole"]
  pub fk_role: i32,
  #[rename = "fkperson"]
  pub fk_person: i32,
}
