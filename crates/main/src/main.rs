//! Main entrypoint.

use std::sync::Arc;

use xitca_web::http::{WebRequest, WebResponse};

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let addr: std::net::SocketAddr = envfury::or_parse("ADDR", "0.0.0.0:8080")?;

    tracing::info!(message = "About to start the server", %addr);

    let service = mem_fs_server::MemFsServer::default();
    let service = MemFsServerService(Arc::new(service));

    xitca_web::HttpServer::serve(service)
        .bind(addr)?
        .run()
        .await?;

    Ok(())
}

/// The [`xitca_web`] integration for the [`mem_fs_server::MemFsServer`].
struct MemFsServerService(pub Arc<mem_fs_server::MemFsServer>);

impl xitca_web::service::Service for MemFsServerService {
    type Response = MemFsServerService;
    type Error = std::convert::Infallible;

    async fn call(&self, _req: ()) -> Result<Self::Response, Self::Error> {
        Ok(MemFsServerService(Arc::clone(&self.0)))
    }
}

impl xitca_web::service::ready::ReadyService for MemFsServerService {
    type Ready = ();

    #[inline]
    async fn ready(&self) -> Self::Ready {}
}

impl xitca_web::service::Service<WebRequest> for MemFsServerService {
    type Response = WebResponse;
    type Error = std::convert::Infallible;

    #[inline]
    async fn call(&self, req: WebRequest) -> Result<Self::Response, Self::Error> {
        Ok(self.0.handle_request(req))
    }
}
