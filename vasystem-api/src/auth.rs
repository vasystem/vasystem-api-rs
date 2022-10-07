use std::sync::Arc;
use std::task::{Context, Poll};

use crate::oauth2::TokenSource;
use futures_core::future::BoxFuture;
use http::{HeaderValue, Request};
use tower::Service;

#[derive(Clone)]
pub struct AuthSvc<T> {
    inner: T,
    token_source: Arc<TokenSource>,
}

impl<T> AuthSvc<T> {
    pub fn new(inner: T, token_source: Arc<TokenSource>) -> Self {
        Self {
            inner,
            token_source,
        }
    }
}

impl<T, ReqBody> Service<Request<ReqBody>> for AuthSvc<T>
where
    ReqBody: Send + 'static,
    T: Service<Request<ReqBody>> + Clone + Send + 'static,
    T::Future: Send + 'static,
    T::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = T::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
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
