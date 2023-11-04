use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::oauth2::TokenSource;
use http::{HeaderValue, Request, Response};
use tonic::body::BoxBody;
use tonic::transport::{Body, Channel};
use tower::Service;

#[derive(Clone)]
pub struct AuthSvc {
    inner: Channel,
    token_source: Arc<TokenSource>,
}

impl AuthSvc {
    pub fn new(inner: Channel, token_source: Arc<TokenSource>) -> Self {
        Self {
            inner,
            token_source,
        }
    }
}

impl Service<Request<BoxBody>> for AuthSvc {
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<BoxBody>) -> Self::Future {
        let token_source = self.token_source.clone();

        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let access_token = match token_source.access_token().await {
                Ok(token) => token,
                Err(e) => return Err(e.into()),
            };
            let token: HeaderValue = format!("Bearer {}", access_token).parse().unwrap();
            req.headers_mut().insert("authorization", token);

            let response = match inner.call(req).await {
                Ok(response) => response,
                Err(e) => return Err(e.into()),
            };

            Ok(response)
        })
    }
}
