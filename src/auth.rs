use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::codegen::http::HeaderValue;
use tonic::codegen::Service;
use tonic::metadata::{Ascii, MetadataValue};
use tonic::Request;

#[derive(Debug, Clone)]
pub struct PasswordAuthService<S> {
    inner: S,
    username: Option<Arc<MetadataValue<Ascii>>>,
    password: Option<Arc<MetadataValue<Ascii>>>,
}

impl<S> PasswordAuthService<S>  {
    pub fn new(inner: S, username: Option<Arc<MetadataValue<Ascii>>>, password: Option<Arc<MetadataValue<Ascii>>>) -> Self {
        Self {
            inner,
            username,
            password
        }
    }
}

impl<S, Body, Response> Service<Request<Body>> for PasswordAuthService<S>
where
    S: Service<Request<Body>, Response = Response>
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if let Some(username) = &self.username {
            if let Some(password) = &self.password {
                req.metadata_mut()
                    .insert("username", username.as_ref().clone());
                req.metadata_mut()
                    .insert("password", password.as_ref().clone());
            }
        }

        self.inner.call(req)
    }
}