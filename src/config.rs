use crate::cli::Args;
use crate::error::{Error, Result};
use crate::openrouter::DEFAULT_MODEL;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub verbose: bool,
}

impl Config {
    pub fn load(args: &Args) -> Result<Self> {
        // Load config file if it exists
        let file_config = Self::load_config_file(args.config_path.as_deref())?;

        // Load credentials from ~/.config/youtube-summary/credentials
        let credentials = Credentials::load()?;

        // API key precedence: CLI > env > credentials > config file
        let api_key = args
            .api_key
            .clone()
            .or_else(|| env::var("OPENROUTER_API_KEY").ok())
            .or(credentials.openrouter_api_key)
            .or(file_config.api_key)
            .ok_or_else(|| {
                Error::Config(
                    "No API key found. Set OPENROUTER_API_KEY env var, use --api-key, or add to ~/.config/youtube-summary/credentials"
                        .to_string(),
                )
            })?;

        // Model precedence: CLI > config file > default
        let model = args
            .model
            .clone()
            .or(file_config.model)
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        // Prompt: CLI > default
        let prompt = args.prompt.clone().unwrap_or_else(|| {
            "Please provide a comprehensive summary of the following YouTube video transcript. \
             Include the main topics discussed, key points, and any important conclusions."
                .to_string()
        });

        Ok(Config {
            api_key,
            model,
            prompt,
            verbose: args.verbose,
        })
    }

    fn load_config_file(custom_path: Option<&str>) -> Result<FileConfig> {
        let path = match custom_path {
            Some(p) => PathBuf::from(p),
            None => {
                let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config/youtube-summary/config")
            }
        };

        if !path.exists() {
            return Ok(FileConfig::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        Self::parse_config(&content)
    }

    fn parse_config(content: &str) -> Result<FileConfig> {
        let mut config = FileConfig::default();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "api_key" => config.api_key = Some(value.to_string()),
                    "default_model" => config.model = Some(value.to_string()),
                    _ => {} // Ignore unknown keys
                }
            }
        }

        Ok(config)
    }
}

#[derive(Debug, Default)]
struct FileConfig {
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Debug, Default)]
struct Credentials {
    openrouter_api_key: Option<String>,
}

impl Credentials {
    fn load() -> Result<Self> {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = PathBuf::from(home).join(".config/youtube-summary/credentials");

        if !path.exists() {
            return Ok(Credentials::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("Failed to read credentials file: {}", e)))?;

        let mut credentials = Credentials::default();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                if key == "OPENROUTER_API_KEY" {
                    credentials.openrouter_api_key = Some(value.to_string());
                }
            }
        }

        Ok(credentials)
    }
}
