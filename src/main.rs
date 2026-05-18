#![allow(clippy::collapsible_if)]

use crate::cli::X1Brief;

// use crate::cli::X1Brief;
// mod ai;
mod cli;
mod clipboard;
mod notifier;
mod toml_setup;
// mod selectore;
// mod textprocess;


fn main() -> anyhow::Result<()> {
    let app = X1Brief::new();

    match app.run() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
