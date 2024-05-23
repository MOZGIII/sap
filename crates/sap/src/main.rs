//! Main entrypoint.

use std::sync::Arc;

use xitca_web::http::{WebRequest, WebResponse};

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let addr: std::net::SocketAddr = envfury::or_parse("ADDR", "0.0.0.0:8080")?;

    let root_dir: std::path::PathBuf = envfury::must("ROOT_DIR")?;

    let max_file_size: spa_loader::FileSize =
        envfury::or("MAX_FILE_SIZE", spa_loader::FileSize::MAX)?;

    let mode: Mode = envfury::or_else("MODE", Mode::default)?;

    let root_as_not_found: bool = envfury::or("ROOT_AS_NOT_FOUND", true)?;

    let no_root_templating: bool = envfury::or("NO_ROOT_TEMPLATING", false)?;

    let cfg_env_prefix: String = envfury::or_parse("CFG_ENV_PREFIX", "APP_")?;

    let loader = spa_loader::Loader {
        max_file_size,
        root_dir,
        root_as_not_found,
        root_templating: (!no_root_templating).then_some(spa_cfg_html::Engine {
            env_prefix: std::borrow::Cow::Owned(cfg_env_prefix),
            template_tag_presence: spa_cfg_html::TemplateTagPresence::Required,
        }),
    };

    tracing::info!(message = "Loading the files into memory", ?loader);

    let service = loader.load().await?;

    if mode == Mode::Check {
        return Ok(());
    }

    let service = MemServerService(Arc::new(service));

    tracing::info!(message = "About to start the server", %addr);

    xitca_web::HttpServer::serve(service)
        .bind(addr)?
        .run()
        .await?;

    Ok(())
}

/// The [`xitca_web`] integration for the [`mem_server::MemServer`].
struct MemServerService(pub Arc<mem_server::MemServer>);

impl xitca_web::service::Service for MemServerService {
    type Response = Self;
    type Error = std::convert::Infallible;

    async fn call(&self, _req: ()) -> Result<Self::Response, Self::Error> {
        Ok(Self(Arc::clone(&self.0)))
    }
}

impl xitca_web::service::ready::ReadyService for MemServerService {
    type Ready = ();

    #[inline]
    async fn ready(&self) -> Self::Ready {}
}

impl xitca_web::service::Service<WebRequest> for MemServerService {
    type Response = WebResponse;
    type Error = std::convert::Infallible;

    #[inline]
    async fn call(&self, req: WebRequest) -> Result<Self::Response, Self::Error> {
        Ok(self.0.handle_request(req))
    }
}

/// The operation mode.
#[derive(Debug, PartialEq, Eq, Default, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum Mode {
    /// Run the server.
    #[default]
    Run,
    /// Load the SPA and exit.
    Check,
}
