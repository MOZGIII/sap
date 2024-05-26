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

    let root_templating: RootTemplating =
        envfury::or_else("ROOT_TEMPLATING", RootTemplating::default)?;
    let config_json_templating: bool = envfury::or("CONFIG_JSON_TEMPLATING", false)?;

    let cfg_env_prefix: String = envfury::or_parse("CFG_ENV_PREFIX", "APP_")?;

    let mut global_headers: yaml_headers::Headers = envfury::or_parse("GLOBAL_HEADERS", "")?;
    let global_headers_file: Option<std::path::PathBuf> = envfury::maybe("GLOBAL_HEADERS_FILE")?;

    if let Some(path) = global_headers_file {
        let data = tokio::fs::read_to_string(path).await?;
        let parsed: yaml_headers::Headers = data.parse()?;
        global_headers.0.extend(parsed.0);
    }

    let loader = spa_loader::Loader {
        max_file_size,
        root_dir,
        root_as_not_found,
        root_templating: (!matches!(root_templating, RootTemplating::Disabled)).then_some(
            spa_cfg_html::Engine {
                env_prefix: std::borrow::Cow::Owned(cfg_env_prefix.clone()),
                template_tag_presence: match root_templating {
                    RootTemplating::Auto => spa_cfg_html::TemplateTagPresence::SkipIfNotFound,
                    RootTemplating::Force => spa_cfg_html::TemplateTagPresence::Required,
                    RootTemplating::Disabled => unreachable!(),
                },
            },
        ),
        config_json_templating: config_json_templating.then_some(spa_cfg_json::Engine {
            env_prefix: std::borrow::Cow::Owned(cfg_env_prefix),
        }),
        headers: global_headers.into(),
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

/// The mode of root templating.
#[derive(Debug, PartialEq, Eq, Default, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum RootTemplating {
    /// Templatify if the the script template tag is found.
    #[default]
    Auto,
    /// Require the HTML templating of the root route.
    Force,
    /// Do not attempt templatifying.
    Disabled,
}
