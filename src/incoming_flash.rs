//! Extractor for incoming flash messages.

use crate::{create_cookie, Config, FlashMessage, Level, COOKIE_NAME};
use async_trait::async_trait;
use axum_core::{
    extract::{FromRef, FromRequestParts},
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use axum_extra::extract::cookie::SignedCookieJar;
use cookie::Key;
use http::{request::Parts, StatusCode};
use std::{convert::Infallible, fmt};

/// Extractor for incoming flash messages.
///
/// See [root module docs](crate) for an example.
#[derive(Clone)]
pub struct IncomingFlashes {
    flashes: Vec<FlashMessage>,
    use_secure_cookies: bool,
    key: Key,
}

impl fmt::Debug for IncomingFlashes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IncomingFlashes")
            .field("flashes", &self.flashes)
            .field("use_secure_cookies", &self.use_secure_cookies)
            .field("key", &"REDACTED")
            .finish()
    }
}

impl IncomingFlashes {
    /// Get an iterator over the flash messages.
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.flashes.iter())
    }

    /// Get the number of flash messages.
    pub fn len(&self) -> usize {
        self.flashes.len()
    }

    /// Whether there are any flash messages or not.
    pub fn is_empty(&self) -> bool {
        self.flashes.is_empty()
    }
}

/// An iterator over the flash messages.
#[derive(Debug)]
pub struct Iter<'a>(std::slice::Iter<'a, FlashMessage>);

impl<'a> Iterator for Iter<'a> {
    type Item = (Level, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let message = self.0.next()?;
        Some((message.level, &message.message))
    }
}

impl<'a> IntoIterator for &'a IncomingFlashes {
    type Item = (Level, &'a str);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for IncomingFlashes
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);
        let cookies = SignedCookieJar::from_headers(&parts.headers, config.key.clone());

        let flashes = cookies
            .get(COOKIE_NAME)
            .map(|cookie| cookie.into_owned())
            .and_then(|cookie| serde_json::from_str::<Vec<FlashMessage>>(cookie.value()).ok())
            .unwrap_or_default();

        Ok(Self {
            flashes,
            use_secure_cookies: config.use_secure_cookies,
            key: config.key,
        })
    }
}

impl IntoResponseParts for IncomingFlashes {
    type Error = Infallible;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let cookies = SignedCookieJar::from_headers(res.headers(), self.key);

        let mut cookie = create_cookie("".to_owned(), self.use_secure_cookies);
        cookie.make_removal();
        let cookies = cookies.add(cookie);
        cookies.into_response_parts(res)
    }
}

impl IntoResponse for IncomingFlashes {
    fn into_response(self) -> Response {
        (self, ()).into_response()
    }
}
