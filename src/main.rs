mod cli;
mod config;
mod error;
mod openrouter;
mod transcript;

use std::env;

use cli::Args;
use config::Config;
use error::Error;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> error::Result<()> {
    // Parse CLI arguments
    let args = Args::parse().map_err(|e| {
        eprintln!("{}", e);
        std::process::exit(0);
    })?;

    // Handle --list-models early (only needs API key)
    if let Some(ref search) = args.list_models {
        let api_key = get_api_key(&args)?;
        return openrouter::list_models(&api_key, search.as_deref(), args.verbose).await;
    }

    // Load full configuration for summarization
    let config = Config::load(&args)?;

    // URL is guaranteed to be present here (checked in Args::parse)
    let url = args.url.as_ref().unwrap();

    if config.verbose {
        eprintln!("[verbose] URL: {}", url);
        eprintln!("[verbose] Fetching transcript...");
    }

    // Fetch transcript
    let transcript = transcript::fetch_transcript(url).await?;

    if config.verbose {
        eprintln!("[verbose] Transcript fetched: {} chars", transcript.len());
    }

    // Send to OpenRouter for summarization
    let summary = openrouter::summarize(&config, &transcript).await?;

    // Print the summary
    println!("{}", summary);

    Ok(())
}

/// Get API key for list-models command (simpler than full Config::load)
fn get_api_key(args: &Args) -> error::Result<String> {
    // Try CLI argument first
    if let Some(ref key) = args.api_key {
        return Ok(key.clone());
    }

    // Try environment variable
    if let Ok(key) = env::var("OPENROUTER_API_KEY") {
        return Ok(key);
    }

    // Try credentials file
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let creds_path = std::path::PathBuf::from(home).join(".config/youtube-summary/credentials");

    if creds_path.exists()
        && let Ok(content) = std::fs::read_to_string(&creds_path)
    {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=')
                && key.trim() == "OPENROUTER_API_KEY"
            {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                return Ok(value.to_string());
            }
        }
    }

    Err(Error::Config(
        "No API key found. Set OPENROUTER_API_KEY env var or use --api-key".to_string(),
    ))
}
