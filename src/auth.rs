use std::error::Error;
use http::{HeaderValue, Request};
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::codegen::Body;
use tower_service::Service;

#[derive(Debug, Clone)]
pub struct AuthService<S> {
    inner: S,
    username: Option<Arc<HeaderValue>>,
    password: Option<Arc<HeaderValue>>
}

impl<S> AuthService<S> {
    #[inline]
    pub fn new(inner: S, username: Option<Arc<HeaderValue>>, password: Option<Arc<HeaderValue>>) -> Self {
        Self { inner, username, password }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthService<S>
    where
        S: Service<Request<ReqBody>, Response = ResBody>,
        S::Error: ,
        ResBody: Body,
        <ResBody as Body>::Error: Into<Box<dyn Error + Send + Sync>>

{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        if let Some(user) = &self.username {
            if let Some(pass) = &self.password {
                request
                    .headers_mut()
                    .insert("username", user.as_ref().clone());
                request.headers_mut()
                    .insert("password", pass.as_ref().clone());
            }
        }

        self.inner.call(request)
    }
}