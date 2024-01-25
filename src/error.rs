#[derive(thiserror::Error, Debug)]
pub enum GinmiError {
    #[error("failed to connect to endpoint: {}", .0)]
    ConnectionError(#[from] tonic::transport::Error)
}