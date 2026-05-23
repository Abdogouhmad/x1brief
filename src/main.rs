#![allow(clippy::collapsible_if)]
mod ai;
mod cli;
mod clipboard;
mod notifier;
// mod toml_setup;
mod regex;

use crate::cli::X1Brief;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = X1Brief::new();

    app.run().await?;

    Ok(())
}
