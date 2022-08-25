//! Middleware necessary for `axum_flash` to work.
//!
//! See [root module docs](crate) for an example.

use crate::{
    private::{AddExtensionLayer, UseSecureCookies},
    SigningKey,
};
use cookie::Key;
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
