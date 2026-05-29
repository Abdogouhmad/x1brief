use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use config::Config;
use dirs;
use thiserror::Error;

use crate::ai::{ModelFamily, ModelSize};


/// =========================
/// Top-level config wrapper
/// =========================
#[derive(Deserialize,Serialize, Debug, Clone)]
pub struct X1BriefConfig {
    pub config_path: PathBuf,
    pub model: ModelConfig,
}

/// =========================
/// [model] section in TOML
/// =========================
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ModelConfig {
    pub model: ModelFamily,
    pub size_model: ModelSize,
}


impl Default for ModelConfig {
    fn default() -> Self {
        let model = ModelFamily::Gemma;
        let size_model = ModelSize::Small;
        Self {
            model: model,
            size_model: size_model,
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
    pub fn model(&self) -> &ModelFamily {
        &self.model.model
    }

    pub fn size_model(&self) -> &ModelSize {
        &self.model.size_model
    }

}
