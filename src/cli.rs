use std::{thread::sleep, time::Duration};

use clap::Parser;
// use tokio::sync::watch;

use crate::clipboard::X1BriefClipboard;
use crate::notifier::X1BriefNotifier;
use crate::toml_setup::X1BriefConfig;

/// X1Brief rust based cli that summarizes text using free AI model
#[derive(Parser)]
pub struct X1Brief {
    #[arg(short = 's', long = "sum", num_args = 0..=1)]
    pub sum: Option<String>,
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,
    #[arg(short = 'w', long = "watch")]
    pub watch: bool,
}

/// ENUM for Error types
#[derive(thiserror::Error, Debug)]
pub enum X1briefErrors {
    // unknown argument
    #[error("unknown argument")]
    ArgsNotFound,
    // Missing required argument
    // #[error("required argument not found")]
    // RequiredArgNotFound,
    // empty value
    #[error("Not taking an empty value")]
    EmptyValue,
}


impl X1Brief {

    /// Parses the CLI arguments
    pub fn new() -> Self {
        Self::parse()
    }

    /// Runs the CLI application
    pub fn run(&self) -> anyhow::Result<()> {
        self.run_checks()?;
        if self.watch {
            println!("Watch mode is running...");
            self.watch_mode()?;
        };
        if self.debug {
            let toml = X1BriefConfig::load()?;
            println!("Config path: {}", toml.config_path.display());
            println!("Model path: {}", toml.model_path());
            println!("Tokenizer path: {}", toml.tokenizer_path());
        };

        if let Some(s) = &self.sum {
            println!("Sum: {}", s);
        }
        Ok(())
    }

    /// Runs checks on the parsed arguments
    fn run_checks(&self) -> anyhow::Result<()> {
        if self.watch && self.debug {
            return Err(X1briefErrors::ArgsNotFound.into());
        }

        match &self.sum {
            Some(s) => {
                if s.trim().is_empty() {
                    return Err(X1briefErrors::EmptyValue.into());
                }
            }
            None => {
                // return Err(X1briefErrors::EmptyValue.into());
            }
        }

        Ok(())
    }

    /// Runs the watch mode, monitoring the clipboard for changes & summarizing the contents
    fn watch_mode(&self) -> anyhow::Result<()> {
        let mut clipboard = X1BriefClipboard::new()?;
        let mut last_text = clipboard.get_text()?;
        let notifier = X1BriefNotifier::new();

        notifier.notify("X1Brief", "The app is launching in watch mode");

        while self.watch {
            sleep(Duration::from_millis(500));

            let current_text = clipboard.get_text()?;

            if current_text != last_text {
                println!("Clipboard changed: {}", current_text);
                println!("Clipboard changed: {}", current_text.len());
                // TODO: call AI summarizer here
                last_text = current_text;
            }
        }

        Ok(())
    }
}
