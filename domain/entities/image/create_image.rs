#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateImage<'a> {
  pub file_name: &'a str,
  pub file_path: &'a str,
  pub uri: &'a str,
  pub display_path: &'a str,
}
