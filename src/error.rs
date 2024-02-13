#[derive(thiserror::Error, Debug)]
pub enum GinmiError {
    #[error("error connecting to endpoint: {}", .0)]
    TransportError(#[from] tonic::transport::Error),
    #[error("invalid uri passed as target: {}", .0)]
    InvalidUriError(String),
}