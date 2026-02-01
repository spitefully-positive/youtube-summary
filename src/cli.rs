use std::env;

use crate::openrouter::DEFAULT_MODEL;

#[derive(Debug)]
pub struct Args {
    pub url: Option<String>,
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub config_path: Option<String>,
    pub verbose: bool,
    pub list_models: Option<Option<String>>,
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            return Err(Self::usage());
        }

        // Check for help flag
        if args.iter().any(|a| a == "--help" || a == "-h") {
            return Err(Self::usage());
        }

        let mut url = None;
        let mut prompt = None;
        let mut model = None;
        let mut api_key = None;
        let mut config_path = None;
        let mut verbose = false;
        let mut list_models: Option<Option<String>> = None;

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];

            match arg.as_str() {
                "-p" | "--prompt" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--prompt requires a value".to_string());
                    }
                    prompt = Some(args[i].clone());
                }
                "-m" | "--model" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--model requires a value".to_string());
                    }
                    model = Some(args[i].clone());
                }
                "-k" | "--api-key" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--api-key requires a value".to_string());
                    }
                    api_key = Some(args[i].clone());
                }
                "-c" | "--config" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--config requires a path".to_string());
                    }
                    config_path = Some(args[i].clone());
                }
                "-v" | "--verbose" => {
                    verbose = true;
                }
                "-l" | "--list-models" => {
                    // Check if next arg is a search term (not starting with -)
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        i += 1;
                        list_models = Some(Some(args[i].clone()));
                    } else {
                        list_models = Some(None);
                    }
                }
                _ if !arg.starts_with('-') && url.is_none() => {
                    url = Some(arg.clone());
                }
                _ => {
                    return Err(format!("Unknown argument: {}", arg));
                }
            }
            i += 1;
        }

        // URL is required unless --list-models is specified
        if list_models.is_none() && url.is_none() {
            return Err("YouTube URL is required".to_string());
        }

        Ok(Args {
            url,
            prompt,
            model,
            api_key,
            config_path,
            verbose,
            list_models,
        })
    }

    fn usage() -> String {
        format!(
            r#"Usage: youtube-summary [OPTIONS] [URL]

Arguments:
  [URL]                     YouTube video URL (required unless --list-models)

Options:
  -p, --prompt <PROMPT>     Custom prompt for the summary
  -m, --model <MODEL>       OpenRouter model ID (default: {})
  -k, --api-key <KEY>       OpenRouter API key (overrides env/config)
  -c, --config <PATH>       Path to config file
  -l, --list-models [TERM]  List available models (optionally filter by TERM)
  -v, --verbose             Show verbose output
  -h, --help                Show this help message

Environment:
  OPENROUTER_API_KEY        API key for OpenRouter

Examples:
  youtube-summary "https://youtube.com/watch?v=VIDEO_ID"
  youtube-summary "https://youtube.com/watch?v=VIDEO_ID" -m anthropic/claude-sonnet-4
  youtube-summary --list-models                    # List all models
  youtube-summary --list-models claude             # List models matching "claude"
  youtube-summary -l gpt -v                        # List GPT models with verbose output"#,
            DEFAULT_MODEL
        )
    }
}
