use std::sync::Arc;

use clap::Parser;
use tokio::{
    sync::Mutex,
    time::{Duration, sleep},
};

use crate::{
    ai::{ModelFamily, ModelSize, X1Ai},
    clipboard::X1BriefClipboard,
    notifier::X1BriefNotifier,
    regex::X1BriefRegex, toml_setup::X1BriefConfig,
};

/// X1Brief rust based cli that summarizes text using free AI model
#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct X1Brief {
    /// Summarize the given text
    #[arg(short = 's', long = "sum")]
    pub sum: Option<String>,

    /// Enable debug mode
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Watch clipboard continuously
    #[arg(short = 'w', long = "watch")]
    pub watch: bool,

    /// AI model family
    #[arg(long, value_enum, default_value = "gemma")]
    pub prompted_model: ModelFamily,

    /// AI model size
    #[arg(long, value_enum, default_value = "small")]
    pub prompted_size_model: ModelSize,
}

/// CLI errors
#[derive(thiserror::Error, Debug)]
pub enum X1briefErrors {
    #[error("watch and debug modes cannot run together")]
    InvalidModes,

    #[error("empty text is not allowed")]
    EmptyValue,
}

impl X1Brief {
    /// Parse CLI arguments
    pub fn new() -> Self {
        Self::parse()
    }

    /// Run application
    pub async fn run(&self) -> anyhow::Result<()> {
        self.run_checks()?;

        // ---------------- DEBUG ----------------

        if self.debug {
            let toml_conf = X1BriefConfig::load()?;
            let def_model = toml_conf.model();
            let def_size = toml_conf.size_model();

            println!("You are calling {:?} at size of {:?}", def_model, def_size);
            // let checker = X1BriefRegex::new()?;

            // println!("Password Tests:");
            // println!("hello => {}", checker.is_password("hello"));
            // println!("Hello123! => {}", checker.is_password("Hello123!"));
        }

        // ---------------- DIRECT SUMMARIZE ----------------

        if let Some(text) = &self.sum {
            let mut ai = X1Ai::new(self.prompted_model, self.prompted_size_model)?;

            println!("Generating summary...\n");

            let prompt = format!(
                "You are an AI assistant that summarizes text: {}",
                text
            );

            let summary = ai.generate(&prompt)?;

            println!("================ SUMMARY ================\n");
            println!("{summary}\n");
        }

        // ---------------- WATCH MODE ----------------

        if self.watch {
            println!("Watch mode started...");
            self.watch_mode().await?;
        }

        Ok(())
    }

    /// Validate CLI arguments
    fn run_checks(&self) -> anyhow::Result<()> {
        if self.watch && self.debug {
            return Err(X1briefErrors::InvalidModes.into());
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
        let regex = X1BriefRegex::new()?;

        // Lazy AI initialization
        let mut ai: Option<Arc<Mutex<X1Ai>>> = None;

        notifier.notify("X1Brief", "Clipboard watch mode started");

        loop {
            sleep(Duration::from_millis(500)).await;

            // ---------------- CLIPBOARD ----------------

            let current_text = match clipboard.get_text() {
                Ok(text) => text,
                Err(_) => continue,
            };

            // Skip identical text
            if current_text == last_text {
                continue;
            }

            last_text = current_text.clone();

            let text = current_text.trim();

            // ---------------- FAST FILTERS ----------------

            if text.is_empty() {
                continue;
            }

            // Too small or too large
            if text.len() < 10 || text.len() > 5000 {
                continue;
            }

            // Whitespace only
            if text.chars().all(|c| c.is_whitespace()) {
                continue;
            }

            // Password detection
            if regex.is_password(text) {
                eprintln!("🔒 Password detected — stopping watch mode");
                break;
            }

            println!("Clipboard changed ({} chars)", text.len());

            // ---------------- LAZY AI LOAD ----------------

            if ai.is_none() {
                println!("Loading AI model: {:?} {:?}", self.prompted_model, self.prompted_size_model);

                ai = Some(Arc::new(Mutex::new(X1Ai::new(self.prompted_model, self.prompted_size_model)?)));
            }

            let ai = ai.as_ref().unwrap().clone();
            let text = text.to_string();
            let prompt = format!(
                "You are an AI assistant that summarizes text: {}",
                text
            );

            // ---------------- INFERENCE ----------------

            let summary = tokio::task::spawn_blocking(move || {
                let mut ai = ai.blocking_lock();
                ai.generate(&prompt)
            })
            .await??;

            println!("\n============== SUMMARY ==============\n");
            println!("{summary}\n");

            notifier.notify("X1Brief Summary", "Your summary is ready");
        }

        Ok(())
    }
}
