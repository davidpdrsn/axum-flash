//! Private types that are part of the public. Ideally users shouldn't have to
//! name these but we can expose them upon request.

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
};
use std::fmt;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct UseSecureCookies(pub(crate) bool);

#[derive(Clone)]
pub struct SigningKey(pub(crate) cookie::Key);

impl fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SigningKey").finish()
    }
}

#[async_trait]
impl<B> FromRequest<B> for SigningKey
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(signing_key) =
            Extension::<SigningKey>::from_request(req)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "`SigningKey` extension missing. Did you forget to add `axum_flash::layer()` to your `axum::Router`?",
                    )
                })?;

        Ok(signing_key)
    }
}
