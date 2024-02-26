use tonic::metadata::AsciiMetadataValue;
use tonic::{Request, Status};
use tonic::service::Interceptor;

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    username: AsciiMetadataValue,
    password: AsciiMetadataValue,
}

impl AuthInterceptor {
    pub fn new(username: Option<AsciiMetadataValue>, password: Option<AsciiMetadataValue>) -> Self {
        Self {
            username: username.unwrap_or(AsciiMetadataValue::from_static("")),
            password: password.unwrap_or(AsciiMetadataValue::from_static(""))
        }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.metadata_mut()
            .insert("username", self.username.clone());
        request.metadata_mut()
            .insert("password", self.password.clone());
        Ok(request)
    }
}