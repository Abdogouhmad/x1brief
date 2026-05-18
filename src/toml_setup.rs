use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use config::Config;
use dirs;
use thiserror::Error;


/// =========================
/// Top-level config wrapper
/// =========================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct X1BriefConfig {
    pub config_path: PathBuf,
    pub model: ModelConfig,
}

/// =========================
/// [model] section in TOML
/// =========================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelConfig {
    pub model_path: String,
    pub tokenizer_path: String,
}


impl Default for ModelConfig {
    fn default() -> Self {
        let config_dir = dirs::config_dir().unwrap_or_default();
        let model_path = config_dir.join("x1brief").join("models").join("modelname.gguf");
        let tokenizer_path = config_dir.join("x1brief").join("models").join("tokenizer.json");
        Self {
            model_path: model_path.to_string_lossy().into_owned(),
            tokenizer_path: tokenizer_path.to_string_lossy().into_owned(),
        }
    }
}


#[derive(Error, Debug)]
pub enum X1ErrorConfig {
    #[error("config directory not found")]
    ConfigDirNotFound,

    #[error("failed to serialize default config")]
    SerializeError,

    #[error("failed to deserialize config")]
    DeserializeError,
}


impl X1BriefConfig {
    /// Load config OR create default file
    pub fn load() -> Result<Self, X1ErrorConfig> {
        let config_dir = dirs::config_dir()
            .ok_or(X1ErrorConfig::ConfigDirNotFound)?;

        let config_path = config_dir
            .join("x1brief")
            .join("config.toml");

        // ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Create default config file
        if !config_path.exists() {
            let default = X1BriefConfig {
                config_path: config_path.clone(),
                model: ModelConfig::default(),
            };

            let toml_string = toml::to_string_pretty(&default)
                .map_err(|_| X1ErrorConfig::SerializeError)?;

            fs::write(&config_path, toml_string).ok();

            return Ok(default);
        }

        // =========================
        // Load existing config file
        // =========================
        let settings = Config::builder()
            .add_source(config::File::from(config_path.clone()))
            .build()
            .map_err(|_| X1ErrorConfig::DeserializeError)?;

        let model: ModelConfig = settings
            .get::<ModelConfig>("model")
            .map_err(|_| X1ErrorConfig::DeserializeError)?;

        Ok(Self {
            config_path,
            model,
        })
    }

    /// helper getters
    pub fn model_path(&self) -> &str {
        &self.model.model_path
    }

    pub fn tokenizer_path(&self) -> &str {
        &self.model.tokenizer_path
    }
}
