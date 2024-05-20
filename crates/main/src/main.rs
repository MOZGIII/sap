//! Main entrypoint.

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    println!("Hello, world!");

    Ok(())
}
