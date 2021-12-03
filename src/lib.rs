//! One-time notifications (aka flash messages) for [axum].
//!
//! Flash messages are stored in signed cookies managed by [tower-cookies]. This
//! means if your app is otherwise also using cookies those should be managed by
//! [tower-cookies] as well since it overrides response headers.
//!
//! # Example
//!
//! ```
//! use axum::{
//!     response::{IntoResponse, Redirect},
//!     routing::get,
//!     Router,
//! };
//! use axum_flash::{IncomingFlashes, Flash, Key};
//!
//! // This should probably come from configuration
//! let key = Key::generate();
//!
//! let app = Router::new()
//!     .route("/", get(root))
//!     .route("/set-flash", get(set_flash))
//!     .layer(axum_flash::layer(key).with_cookie_manager());
//!
//! async fn root(flash: IncomingFlashes) -> impl IntoResponse {
//!     for (level, text) in flash {
//!         // ...
//!     }
//! }
//!
//! async fn set_flash(mut flash: Flash) -> impl IntoResponse {
//!     flash.debug("Hi from flash!");
//!     Redirect::to("/".parse().unwrap())
//! }
//! # async {
//! # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
//! # };
//! ```
//!
//! [axum]: https://crates.io/crates/axum
//! [tower-cookies]: https://crates.io/crates/tower-cookies

#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations,
    missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

use self::private::SigningKey;
use async_trait::async_trait;
use axum_core::extract::{FromRequest, RequestParts};
use http::StatusCode;
use percent_encoding::AsciiSet;
use private::UseSecureCookies;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, time::Duration};
use tower_cookies::{Cookie, Cookies};

#[doc(inline)]
pub use self::{incoming_flash::IncomingFlashes, layer::layer};
#[doc(no_inline)]
pub use cookie::Key;

pub mod incoming_flash;
pub mod layer;

mod private;

/// Extractor for setting outgoing flash messages.
///
/// The flashes will be stored in a signed cookie.
#[derive(Debug)]
pub struct Flash {
    flashes: Vec<FlashMessage>,
    signing_key: SigningKey,
    use_secure_cookies: bool,
    cookies: Cookies,
}

impl Flash {
    /// Push an `Debug` flash message.
    pub fn debug(&mut self, message: impl Into<String>) {
        self.push(Level::Debug, message)
    }

    /// Push an `Info` flash message.
    pub fn info(&mut self, message: impl Into<String>) {
        self.push(Level::Info, message)
    }

    /// Push an `Success` flash message.
    pub fn success(&mut self, message: impl Into<String>) {
        self.push(Level::Success, message)
    }

    /// Push an `Warning` flash message.
    pub fn warning(&mut self, message: impl Into<String>) {
        self.push(Level::Warning, message)
    }

    /// Push an `Error` flash message.
    pub fn error(&mut self, message: impl Into<String>) {
        self.push(Level::Error, message)
    }

    /// Push a flash message with the given level and message.
    pub fn push(&mut self, level: Level, message: impl Into<String>) {
        self.flashes.push(FlashMessage {
            message: message.into(),
            level,
        });
    }
}

#[async_trait]
impl<B> FromRequest<B> for Flash
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request(req).await?;
        let signing_key = SigningKey::from_request(req).await?;

        let use_secure_cookies = if let Some(private::UseSecureCookies(value)) = req
            .extensions()
            .and_then(|ext| ext.get::<UseSecureCookies>().copied())
        {
            value
        } else {
            true
        };

        Ok(Self {
            cookies,
            signing_key,
            use_secure_cookies,
            flashes: Default::default(),
        })
    }
}

const COOKIE_NAME: &str = "axum-flash";

impl Drop for Flash {
    fn drop(&mut self) {
        let json =
            serde_json::to_string(&self.flashes).expect("failed to serialize flash messages");

        // process is inspired by
        // https://github.com/LukeMathWalker/actix-web-flash-messages/blob/main/src/storage/cookies.rs#L54

        let mut jar = cookie::CookieJar::new();
        jar.signed_mut(&self.signing_key.0)
            .add(Cookie::new(COOKIE_NAME, json));
        let signed_cookie = jar.get(COOKIE_NAME).unwrap();
        let signed_value = signed_cookie.value().as_bytes();

        let encoded =
            percent_encoding::percent_encode(signed_value, USERINFO_ENCODE_SET).to_string();

        let cookie = Cookie::build(COOKIE_NAME, encoded)
            // only send the cookie for https (maybe)
            .secure(self.use_secure_cookies)
            // don't allow javascript to access the cookie
            .http_only(true)
            // don't send the cookie to other domains
            .same_site(cookie::SameSite::Strict)
            // allow the cookie for all paths
            .path("/")
            // expire after 10 minutes
            .max_age(
                Duration::from_secs(10 * 60)
                    .try_into()
                    .expect("failed to convert `std::time::Duration` to `time::Duration`"),
            )
            .finish();

        self.cookies.add(cookie);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FlashMessage {
    #[serde(rename = "l")]
    level: Level,
    #[serde(rename = "m")]
    message: String,
}

/// Verbosity level of a flash message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    #[allow(missing_docs)]
    Debug = 0,
    #[allow(missing_docs)]
    Info = 1,
    #[allow(missing_docs)]
    Success = 2,
    #[allow(missing_docs)]
    Warning = 3,
    #[allow(missing_docs)]
    Error = 4,
}

// from
// https://github.com/LukeMathWalker/actix-web-flash-messages/blob/ccd102de31ddbbbca1041416ff670cca1fb7b97a/src/storage/cookies.rs#L173-L196
const FRAGMENT_ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');
const PATH_ENCODE_SET: &AsciiSet = &FRAGMENT_ENCODE_SET.add(b'#').add(b'?').add(b'{').add(b'}');
const USERINFO_ENCODE_SET: &AsciiSet = &PATH_ENCODE_SET
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|')
    .add(b'%');

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use axum::{
        body::Body,
        http::{header, Request},
        response::{IntoResponse, Redirect},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn basic() {
        let key = Key::generate();

        let app = Router::new()
            .route("/", get(root))
            .route("/set-flash", get(set_flash))
            .layer(layer(key).with_cookie_manager());

        async fn root(flash: IncomingFlashes) -> impl IntoResponse {
            flash
                .into_iter()
                .map(|(level, text)| format!("{:?}: {}", level, text))
                .collect::<Vec<_>>()
                .join(", ")
        }

        async fn set_flash(mut flash: Flash) -> impl IntoResponse {
            flash.debug("Hi from flash!");
            Redirect::to("/".parse().unwrap())
        }

        let request = Request::builder()
            .uri("/set-flash")
            .body(Body::empty())
            .unwrap();
        let mut response = app.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_redirection());
        let cookie = response.headers_mut().remove(header::SET_COOKIE).unwrap();

        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();

        assert!(response.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("Max-Age=0"),);

        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Debug: Hi from flash!");
    }
}
