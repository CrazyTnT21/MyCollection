use crate::schemas::db_role_translation::DbRoleTranslation;
use domain::entities::role::Role;
use from_row::FromRow;
use tokio_postgres::Row;

#[derive(FromRow, Debug)]
#[rename = "role"]
pub struct DbRole {
  pub id: i32,
}

impl DbRole {
  pub fn to_entity(self, role_translation: DbRoleTranslation) -> Role {
    Role {
      id: self.id as u32,
      name: role_translation.name,
      language: role_translation.language.into(),
    }
  }
}
