use crate::errors::AppError;

pub(crate) trait ToAppResult
where
    Self: Sized,
{
    async fn to_app_result(self) -> crate::Result<Self>;
}

impl ToAppResult for reqwest::Response {
    async fn to_app_result(self) -> crate::Result<Self> {
        let status = self.status();

        if status.is_client_error() || status.is_server_error() {
            return Err(AppError::http(status, self.text().await?));
        } else {
            return Ok(self);
        }
    }
}
