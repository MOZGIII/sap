//! Main entrypoint.

use std::sync::Arc;

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
    let config_json_templating: bool = envfury::or("CONFIG_JSON_TEMPLATING", false)?;

    let cfg_env_prefix: String = envfury::or_parse("CFG_ENV_PREFIX", "APP_")?;

    let loader = spa_loader::Loader {
        max_file_size,
        root_dir,
        root_as_not_found,
        root_templating: (!no_root_templating).then_some(spa_cfg_html::Engine {
            env_prefix: std::borrow::Cow::Owned(cfg_env_prefix.clone()),
            template_tag_presence: spa_cfg_html::TemplateTagPresence::Required,
        }),
        config_json_templating: config_json_templating.then_some(spa_cfg_json::Engine {
            env_prefix: std::borrow::Cow::Owned(cfg_env_prefix),
        }),
    };

    tracing::info!(message = "Loading the files into memory", ?loader);

    let service = loader.load().await?;

    if mode == Mode::Check {
        return Ok(());
    }

    let service = xitca_mem_server::Service(Arc::new(service));

    tracing::info!(message = "About to start the server", %addr);

    xitca_web::HttpServer::serve(service)
        .bind(addr)?
        .run()
        .await?;

    Ok(())
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
