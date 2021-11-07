//! Middleware necessary for `axum_flash` to work.
//!
//! See [root module docs](crate) for an example.

use crate::{private::UseSecureCookies, SigningKey};
use axum::AddExtensionLayer;
use cookie::Key;
use tower_cookies::CookieManagerLayer;
use tower_layer::{Layer, Stack};

/// [`Layer`] that applies the necessary middleware for `axum_flash` to work.
///
/// See [`LayerBuilder`] for different configuration options and see the [root
/// module docs](crate) for an example.
pub fn layer(signing_key: Key) -> LayerBuilder<AddExtensionLayer<SigningKey>> {
    LayerBuilder {
        layer: AddExtensionLayer::new(SigningKey(signing_key)),
    }
}

/// [`Layer`] that applies the necessary middleware for `axum_flash` to work.
///
/// Constructed with [`layer`].
#[derive(Debug, Clone)]
pub struct LayerBuilder<L> {
    layer: L,
}

impl<L> LayerBuilder<L> {
    /// Also add a [`CookieManagerLayer`] to the middleware stack.
    ///
    /// A [`CookieManagerLayer`] is required for `axum_flash` to work. If you're
    /// manually adding a [`CookieManagerLayer`] elsewhere in your middleware
    /// stack you don't have to call this method, otherwise you do.
    pub fn with_cookie_manager(self) -> LayerBuilder<Stack<CookieManagerLayer, L>> {
        self.push(CookieManagerLayer::new())
    }

    /// Mark the cookie as secure so the cookie will only be sent on `https`.
    ///
    /// Defaults to marking cookies as secure.
    ///
    /// For local development, depending on your brwoser, you might have to set
    /// this to `false` for flash messages to show up.
    ///
    /// See [mdn] for more details on secure cookies.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie
    pub fn use_secure_cookies(
        self,
        use_secure_cookies: bool,
    ) -> LayerBuilder<Stack<AddExtensionLayer<UseSecureCookies>, L>> {
        self.push(AddExtensionLayer::new(UseSecureCookies(use_secure_cookies)))
    }

    fn push<L2>(self, layer: L2) -> LayerBuilder<Stack<L2, L>> {
        LayerBuilder {
            layer: Stack::new(layer, self.layer),
        }
    }
}

impl<L, S> Layer<S> for LayerBuilder<L>
where
    L: Layer<S>,
{
    type Service = L::Service;

    fn layer(&self, inner: S) -> Self::Service {
        self.layer.layer(inner)
    }
}
