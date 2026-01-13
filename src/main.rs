mod anthropic;
mod cli;
mod config;
mod error;
mod transcript;

use cli::Args;
use config::Config;

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

    // Load configuration
    let config = Config::load(&args)?;

    if config.verbose {
        eprintln!("[verbose] URL: {}", args.url);
        eprintln!("[verbose] Fetching transcript...");
    }

    // Fetch transcript
    let transcript = transcript::fetch_transcript(&args.url).await?;

    if config.verbose {
        eprintln!("[verbose] Transcript fetched: {} chars", transcript.len());
    }

    // Send to Anthropic for summarization
    let summary = anthropic::summarize(&config, &transcript).await?;

    // Print the summary
    println!("{}", summary);

    Ok(())
}
