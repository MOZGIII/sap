//! Main entrypoint.

use xitca_web::{handler::handler_service, route::*};

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let addr: std::net::SocketAddr = envfury::or_parse("ADDR", "0.0.0.0:8080")?;

    tracing::info!(message = "About to start the server", %addr);

    xitca_web::App::new()
        .at("/", get(handler_service(|| async { "Hello, world!" })))
        .serve()
        .bind(addr)?
        .run()
        .await?;

    Ok(())
}
