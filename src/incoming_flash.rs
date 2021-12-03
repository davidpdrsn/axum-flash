//! Extractor for incoming flash messages.

use crate::{private::SigningKey, FlashMessage, Level, COOKIE_NAME};
use async_trait::async_trait;
use axum_core::extract::{FromRequest, RequestParts};
use cookie::CookieJar;
use http::StatusCode;
use tower_cookies::{Cookie, Cookies};

/// Extractor for incoming flash messages.
///
/// See [root module docs](crate) for an example.
#[derive(Debug)]
pub struct IncomingFlashes {
    flashes: Vec<FlashMessage>,
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
impl<B> FromRequest<B> for IncomingFlashes
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request(req).await?;
        let SigningKey(signing_key) = SigningKey::from_request(req).await?;

        // process is inspired by
        // https://github.com/LukeMathWalker/actix-web-flash-messages/blob/main/src/storage/cookies.rs#L87

        let flashes = cookies
            .get(COOKIE_NAME)
            .map(Cookie::into_owned)
            .and_then(|cookie| {
                let mut cookie_jar = CookieJar::new();
                cookie_jar.add_original(cookie);
                cookie_jar.signed(&signing_key).get(COOKIE_NAME)
            })
            .and_then(|cookie| serde_json::from_str::<Vec<FlashMessage>>(cookie.value()).ok())
            .unwrap_or_default();

        cookies.remove(Cookie::named(COOKIE_NAME));

        Ok(Self { flashes })
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
