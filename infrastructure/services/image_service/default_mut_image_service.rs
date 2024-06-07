use async_trait::async_trait;

use domain::entities::image::create_image::CreateImage;
use domain::entities::image::Image;
use domain::entities::image::partial_create_image::PartialCreateImage;
use repositories::image_repository::mut_image_repository::MutImageRepository;
use services::file_service::mut_file_service::MutFileService;
use services::image_service::mut_image_service::MutImageService;
use services::traits::service_error::ServiceError;

use crate::services::map_server_error;

pub struct DefaultMutImageService<'a> {
  mut_image_repository: &'a dyn MutImageRepository,
  mut_file_service: &'a dyn MutFileService,
  display_path: &'a str,
  path: &'a str,
}

impl<'a> DefaultMutImageService<'a> {
  pub fn new(mut_image_repository: &'a dyn MutImageRepository,
             mut_file_service: &'a dyn MutFileService,
             display_path: &'a str,
             path: &'a str, ) -> DefaultMutImageService<'a> {
    DefaultMutImageService { mut_image_repository, mut_file_service, display_path, path }
  }
}

#[async_trait]
impl<'a> MutImageService for DefaultMutImageService<'a> {
  async fn create(&self, image: PartialCreateImage) -> Result<Image, ServiceError> {
    //TODO: Validate data size
    let file = self.mut_file_service.create_base64(&image.data.0, self.path, None).await?;
    let image = CreateImage { file_path: self.path, uri: &file.uri, file_name: &file.name, display_path: self.display_path };
    self.mut_image_repository.create(image).await.map_err(map_server_error)
  }
}
