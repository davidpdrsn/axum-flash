//! Private types that are part of the public. Ideally users shouldn't have to
//! name these but we can expose them upon request.

use async_trait::async_trait;
use axum_core::extract::FromRequestParts;
use axum_extra::extract::cookie::Key;
use http::{request::Parts, Request, StatusCode};
use std::fmt;
use std::task::{Context, Poll};
use tower_layer::Layer;
use tower_service::Service;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct UseSecureCookies(pub(crate) bool);

#[derive(Clone)]
pub struct SigningKey(pub(crate) Key);

impl fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SigningKey").finish()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SigningKey
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<SigningKey>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "`SigningKey` extension missing. Did you forget to add `axum_flash::layer()` to your `axum::Router`?",
        ))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AddExtensionLayer<T> {
    value: T,
}

impl<T> AddExtensionLayer<T> {
    pub fn new(value: T) -> Self {
        AddExtensionLayer { value }
    }
}

impl<S, T> Layer<S> for AddExtensionLayer<T>
where
    T: Clone,
{
    type Service = AddExtension<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        AddExtension {
            inner,
            value: self.value.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AddExtension<S, T> {
    inner: S,
    value: T,
}

impl<ResBody, S, T> Service<Request<ResBody>> for AddExtension<S, T>
where
    S: Service<Request<ResBody>>,
    T: Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ResBody>) -> Self::Future {
        req.extensions_mut().insert(self.value.clone());
        self.inner.call(req)
    }
}
