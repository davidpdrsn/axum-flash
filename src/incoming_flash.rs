//! Extractor for incoming flash messages.

use crate::{private::SigningKey, FlashMessage, Level, UseSecureCookies, COOKIE_NAME};
use async_trait::async_trait;
use axum_core::{
    extract::FromRequestParts,
    response::{IntoResponseParts, ResponseParts},
};
use axum_extra::extract::cookie::SignedCookieJar;
use http::{request::Parts, StatusCode};
use std::convert::Infallible;

/// Extractor for incoming flash messages.
///
/// See [root module docs](crate) for an example.
#[derive(Debug)]
pub struct IncomingFlashes {
    flashes: Vec<FlashMessage>,
    use_secure_cookies: bool,
    signing_key: SigningKey,
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

impl IntoIterator for IncomingFlashes {
    type Item = (Level, String);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.flashes.into_iter())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for IncomingFlashes
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let signing_key = SigningKey::from_request_parts(parts, &state).await?;
        let cookies = SignedCookieJar::from_headers(&parts.headers, signing_key.0.clone());

        // process is inspired by
        // https://github.com/LukeMathWalker/actix-web-flash-messages/blob/main/src/storage/cookies.rs#L87
        let use_secure_cookies = if let Some(UseSecureCookies(value)) =
            parts.extensions.get::<UseSecureCookies>().copied()
        {
            value
        } else {
            true
        };

        let flashes = cookies
            .get(COOKIE_NAME)
            .map(|cookie| cookie.into_owned())
            .and_then(|cookie| serde_json::from_str::<Vec<FlashMessage>>(cookie.value()).ok())
            .unwrap_or_default();

        Ok(Self {
            flashes,
            use_secure_cookies,
            signing_key,
        })
    }
}

impl IntoResponseParts for IncomingFlashes {
    type Error = Infallible;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let cookies = SignedCookieJar::from_headers(res.headers(), self.signing_key.0);

        // If it exists then it means Flash executed its into_response_parts, so we ignore this.
        if cookies.get(COOKIE_NAME).is_none() {
            let cookies = cookies.add(crate::create_cookie("".to_owned(), self.use_secure_cookies));
            cookies.into_response_parts(res)
        } else {
            Ok(res)
        }
    }
}

/// Iterator of incoming flash messages.
///
/// Created with `IncomingFlash::into_iter`.
#[derive(Debug)]
pub struct IntoIter(std::vec::IntoIter<FlashMessage>);

impl Iterator for IntoIter {
    type Item = (Level, String);

    fn next(&mut self) -> Option<Self::Item> {
        let message = self.0.next()?;
        Some((message.level, message.message))
    }
}
