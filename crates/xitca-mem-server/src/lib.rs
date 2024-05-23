//! The [`xitca_web`] integration for the [`mem_server::MemServer`].

use std::sync::Arc;

use xitca_web::http::{WebRequest, WebResponse};

/// The [`xitca_web`] integration for the [`mem_server::MemServer`].
pub struct Service(pub Arc<mem_server::MemServer>);

impl xitca_web::service::Service for Service {
    type Response = Self;
    type Error = std::convert::Infallible;

    async fn call(&self, _req: ()) -> Result<Self::Response, Self::Error> {
        Ok(Self(Arc::clone(&self.0)))
    }
}

impl xitca_web::service::ready::ReadyService for Service {
    type Ready = ();

    #[inline]
    async fn ready(&self) -> Self::Ready {}
}

impl xitca_web::service::Service<WebRequest> for Service {
    type Response = WebResponse;
    type Error = std::convert::Infallible;

    #[inline]
    async fn call(&self, req: WebRequest) -> Result<Self::Response, Self::Error> {
        Ok(self.0.handle_request(req))
    }
}
