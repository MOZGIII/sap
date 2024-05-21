//! Main entrypoint.

use xitca_web::{handler::handler_service, route::*};

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    xitca_web::App::new()
        .at("/", get(handler_service(|| async { "Hello, world!" })))
        .serve()
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}
