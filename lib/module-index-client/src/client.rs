use ulid::Ulid;
use url::Url;

use crate::types::ModuleRejectionResponse;
use crate::{IndexClientResult, ModuleDetailsResponse};

#[derive(Debug, Clone)]
pub struct IndexClient {
    base_url: Url,
    auth_token: String,
}

impl IndexClient {
    pub fn new(base_url: Url, auth_token: &str) -> Self {
        Self {
            base_url,
            auth_token: auth_token.to_owned(),
        }
    }

    pub async fn reject_module(
        &self,
        module_id: Ulid,
        rejected_by_display_name: String,
    ) -> IndexClientResult<ModuleRejectionResponse> {
        let reject_url = dbg!(self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("reject"))?;

        let upload_response = reqwest::Client::new()
            .post(reject_url)
            .multipart(
                reqwest::multipart::Form::new().text("rejected by user", rejected_by_display_name),
            )
            .bearer_auth(&self.auth_token)
            .send()
            .await?
            .error_for_status()?;

        Ok(upload_response.json::<ModuleRejectionResponse>().await?)
    }

    pub async fn upload_module(
        &self,
        module_name: &str,
        module_version: &str,
        module_bytes: Vec<u8>,
    ) -> IndexClientResult<ModuleDetailsResponse> {
        let module_upload_part = reqwest::multipart::Part::bytes(module_bytes)
            .file_name(format!("{module_name}_{module_version}.tar"));

        let upload_url = self.base_url.join("modules")?;
        let upload_response = reqwest::Client::new()
            .post(upload_url)
            .multipart(reqwest::multipart::Form::new().part("module bundle", module_upload_part))
            .bearer_auth(&self.auth_token)
            .send()
            .await?
            .error_for_status()?;

        Ok(upload_response.json::<ModuleDetailsResponse>().await?)
    }

    pub async fn download_module(&self, module_id: Ulid) -> IndexClientResult<Vec<u8>> {
        let download_url = dbg!(self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("download"))?;
        let response = dbg!(reqwest::Client::new()
            .get(download_url)
            .bearer_auth(&self.auth_token))
        .send()
        .await?
        .error_for_status()?;

        let bytes = response.bytes().await?;
        dbg!(&bytes.len());

        Ok(bytes.to_vec())
    }
}
