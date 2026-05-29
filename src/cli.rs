use std::sync::Arc;

use clap::Parser;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use crate::{
    ai::{ModelFamily, ModelSize, X1Ai},
    clipboard::X1BriefClipboard,
    notifier::X1BriefNotifier,
    textprocess::X1TextProcess,
    toml_setup::X1BriefConfig,
};

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct X1Brief {
    #[arg(short = 's', long = "sum")]
    pub sum: Option<String>,

    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    #[arg(short = 'w', long = "watch")]
    pub watch: bool,

    #[arg(long, value_enum, default_value = "gemma")]
    pub prompted_model: ModelFamily,

    #[arg(long, value_enum, default_value = "small")]
    pub prompted_size_model: ModelSize,
}

#[derive(thiserror::Error, Debug)]
pub enum X1briefErrors {
    #[error("watch and debug modes cannot run together")]
    InvalidModes,

    #[error("empty text is not allowed")]
    EmptyValue,
}

impl X1Brief {
    pub fn new() -> Self {
        Self::parse()
    }

    fn build_prompt(text: &str) -> String {
        format!(
            "You are an AI assistant that summarizes the following text:\n\n{}",
            text
        )
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        self.run_checks()?;

        if self.debug {
            let config = X1BriefConfig::load()?;

            println!(
                "You are calling {:?} at size {:?}",
                config.model(),
                config.size_model()
            );
        }

        if let Some(text) = &self.sum {
            let processor = X1TextProcess::new()?;
            let text = processor.sanitize(text);

            if !processor.is_valid_text(&text, 5000) {
                return Err(X1briefErrors::EmptyValue.into());
            }

            if processor.is_password(&text) {
                anyhow::bail!("password detected");
            }

            let mut ai =
                X1Ai::new(self.prompted_model, self.prompted_size_model)?;

            println!("Generating summary...\n");

            let summary =
                ai.generate(&Self::build_prompt(&text))?;

            println!("================ SUMMARY ================\n");
            println!("{summary}\n");
        }

        if self.watch {
            println!("Watch mode started...");
            self.watch_mode().await?;
        }

        Ok(())
    }

    fn run_checks(&self) -> anyhow::Result<()> {
        if self.watch && self.debug {
            return Err(X1briefErrors::InvalidModes.into());
        }

        Ok(())
    }

    async fn watch_mode(&self) -> anyhow::Result<()> {
        let mut clipboard = X1BriefClipboard::new()?;
        let mut last_text = clipboard.get_text()?;

        let notifier = X1BriefNotifier::new();
        let processor = X1TextProcess::new()?;

        let mut ai: Option<Arc<Mutex<X1Ai>>> = None;

        notifier.notify(
            "X1Brief",
            "Clipboard watch mode started",
        );

        loop {
            sleep(Duration::from_millis(500)).await;

            let current_text = match clipboard.get_text() {
                Ok(text) => text,
                Err(_) => continue,
            };

            if current_text == last_text {
                continue;
            }

            last_text = current_text.clone();

            let text = processor.sanitize(&current_text);

            if !processor.is_valid_text(&text, 5000) {
                continue;
            }

            if processor.is_password(&text) {
                eprintln!(
                    "🔒 Password detected — stopping watch mode"
                );
                break;
            }

            println!(
                "Clipboard changed ({} chars)",
                text.len()
            );

            if ai.is_none() {
                println!(
                    "Loading AI model: {:?} {:?}",
                    self.prompted_model,
                    self.prompted_size_model
                );

                ai = Some(Arc::new(Mutex::new(
                    X1Ai::new(
                        self.prompted_model,
                        self.prompted_size_model,
                    )?,
                )));
            }

            let ai = ai.as_ref().unwrap().clone();
            let prompt = Self::build_prompt(&text);

            let summary = tokio::task::spawn_blocking(move || {
                let mut ai = ai.blocking_lock();
                ai.generate(&prompt)
            })
            .await??;

            println!("\n============== SUMMARY ==============\n");
            println!("{summary}\n");

            notifier.notify(
                "X1Brief Summary",
                "Your summary is ready",
            );
        }

        Ok(())
    }
}
