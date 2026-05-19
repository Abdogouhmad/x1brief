use std::sync::Arc;

use clap::Parser;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use crate::{ai::X1Ai, regex::X1BriefRegex};
use crate::clipboard::X1BriefClipboard;
use crate::notifier::X1BriefNotifier;
use crate::toml_setup::X1BriefConfig;

/// X1Brief rust based cli that summarizes text using free AI model
#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct X1Brief {
    /// Summarize the given text
    #[arg(short = 's', long = "sum", num_args = 0..=1)]
    pub sum: Option<String>,

    /// Enable debug mode for development
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Watch mode for continuous summarization
    #[arg(short = 'w', long = "watch")]
    pub watch: bool,
}

/// CLI errors
#[derive(thiserror::Error, Debug)]
pub enum X1briefErrors {
    #[error("watch and debug modes cannot run together")]
    ArgsNotFound,

    #[error("Not taking an empty value")]
    EmptyValue,
}

impl X1Brief {
    /// Parse CLI args
    pub fn new() -> Self {
        Self::parse()
    }

    /// Run application
    pub async fn run(&self) -> anyhow::Result<()> {
        self.run_checks()?;

        if self.debug {
            let checker = X1BriefRegex::new()?;

            println!("{}", checker.is_password("hello")); // false
            println!("{}", checker.is_password("Hello123!")); // true
        }

        if let Some(s) = &self.sum {
            println!("Input: {}", s);
        }

        if self.watch {
            println!("Watch mode is running...");
            self.watch_mode().await?;
        }

        Ok(())
    }

    /// Validate arguments
    fn run_checks(&self) -> anyhow::Result<()> {
        if self.watch && self.debug {
            return Err(X1briefErrors::ArgsNotFound.into());
        }

        if let Some(s) = &self.sum {
            if s.trim().is_empty() {
                return Err(X1briefErrors::EmptyValue.into());
            }
        }

        Ok(())
    }

    /// Watch clipboard and summarize changes
    async fn watch_mode(&self) -> anyhow::Result<()> {

        let mut clipboard = X1BriefClipboard::new()?;
        let mut last_text = clipboard.get_text()?;

        let notifier = X1BriefNotifier::new();
        let toml = X1BriefConfig::load()?;
        let regex = X1BriefRegex::new()?;

        // 🔥 Lazy AI (GPU not initialized yet)
        let mut ai: Option<Arc<Mutex<X1Ai>>> = None;

        notifier.notify("X1Brief", "Watch mode started");

        loop {
            sleep(Duration::from_millis(500)).await;

            // 1. clipboard change
            let current_text = match clipboard.get_text() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if current_text == last_text {
                continue;
            }

            last_text = current_text.clone();

            let text = current_text.trim();

            // 2. cheap filters (VERY fast path)
            if text.is_empty() {
                continue;
            }

            if text.len() <= 10 || text.len() > 5000 {
                continue;
            }

            if text.chars().all(|c| c.is_whitespace()) {
                continue;
            }

            // password detection (cheap regex)
            if regex.is_password(text) {
                eprintln!("🔒 Password detected → exiting watch mode");
                break;
            }

            // 3. decide: process or skip (final gate)
            let should_process = true;

            if !should_process {
                continue;
            }

            println!("Clipboard changed ({} chars)", text.len());

            // 4. ONLY THEN load AI (lazy GPU init)
            if ai.is_none() {
                ai = Some(Arc::new(Mutex::new(
                    X1Ai::new(
                        toml.model_path(),
                        toml.tokenizer_path(),
                    )?
                )));
            }

            let ai = ai.as_ref().unwrap().clone();
            let text = text.to_string();

            // 5. spawn blocking inference (GPU work)
            let summary = tokio::task::spawn_blocking(move || {
                let mut ai = ai.blocking_lock();
                ai.ai_sumup(&text)
            })
            .await??;

            println!("\nSummary:\n{}\n", summary);

            notifier.notify(
                "X1Brief Summary",
                "Your summary is ready",
            );
        }

        Ok(())
    }
}
