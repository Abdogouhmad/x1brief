#![allow(clippy::collapsible_if)]
mod summarizer;

use anyhow::Result;
use arboard::Clipboard;
use notify_rust::Notification; // Added here
use std::io::{self, Write};
use summarizer::Summarizer;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<()> {
    let model_path = "gemma-3-1b-it-Q4_K_M.gguf";
    let tokenizer_path = "tokenizer.json";

    let mut summarizer = match Summarizer::new(model_path, tokenizer_path) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("⚠️ Warning: AI Summarizer not initialized: {}", e);
            eprintln!(
                "    Place '{}' and '{}' in the root to enable AI.",
                model_path, tokenizer_path
            );
            None
        }
    };

    // Grab initial text to prime the app so it doesn't trigger on startup
    let mut last_text = Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .unwrap_or_default();

    println!("🚀 X1Brief is active. Copy text (>300 chars) in terminal or browser to test!");

    loop {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(current_text) = clipboard.get_text() {
                // Only trigger if it's genuinely new content and fits your length requirements
                if current_text != last_text && current_text.len() > 300 {
                    // Immediately update last_text BEFORE the heavy async AI work.
                    // This prevents the user from double-triggering if they copy again mid-inference.
                    last_text = current_text.clone();

                    println!(
                        "\n--- New Long Text Detected ({} chars) ---",
                        current_text.len()
                    );

                    let snippet = current_text.chars().take(20).collect::<String>();
                    println!("Snippet: {}...", snippet.replace('\n', " "));

                    if let Some(ref mut s) = summarizer {
                        print!("✨ AI is thinking, processing 1B tokens... ");
                        io::stdout().flush().unwrap();

                        // Send a quick toast so the user knows it's working in the background
                        let _ = Notification::new()
                            .summary("X1Brief")
                            .body("Processing your text...")
                            .icon("dialog-information")
                            .show();

                        match s.summarize(&current_text).await {
                            Ok(summary) => {
                                println!("\n✨ Summary: {}", summary);

                                // Send the summary as a desktop notification
                                let _ = Notification::new()
                                    .summary("✨ AI Summary Ready")
                                    .body(&summary)
                                    .icon("accessories-text-editor")
                                    .show();
                            }
                            Err(e) => {
                                eprintln!("\n❌ Summarization failed: {}", e);

                                let _ = Notification::new()
                                    .summary("❌ X1Brief Error")
                                    .body(&format!("Failed to summarize: {}", e))
                                    .icon("dialog-error")
                                    .show();
                            }
                        }
                    } else {
                        println!("(AI Summarizer not available)");
                    }
                }
            }
        }

        // Poll every 1 second
        sleep(Duration::from_secs(1)).await;
    }
}
