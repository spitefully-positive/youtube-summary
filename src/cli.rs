use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Model {
    Haiku,
    #[default]
    Sonnet,
    Opus,
}

impl Model {
    pub fn api_name(&self) -> &'static str {
        match self {
            Model::Haiku => "claude-3-5-haiku-20241022",
            Model::Sonnet => "claude-sonnet-4-20250514",
            Model::Opus => "claude-opus-4-20250514",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "haiku" => Some(Model::Haiku),
            "sonnet" => Some(Model::Sonnet),
            "opus" => Some(Model::Opus),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Args {
    pub url: String,
    pub prompt: Option<String>,
    pub model: Option<Model>,
    pub api_key: Option<String>,
    pub config_path: Option<String>,
    pub verbose: bool,
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
                        return Err("--model requires a value (haiku, sonnet, opus)".to_string());
                    }
                    model = Some(Model::from_str(&args[i]).ok_or_else(|| {
                        format!("Invalid model: {}. Use haiku, sonnet, or opus", args[i])
                    })?);
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
                _ if !arg.starts_with('-') && url.is_none() => {
                    url = Some(arg.clone());
                }
                _ => {
                    return Err(format!("Unknown argument: {}", arg));
                }
            }
            i += 1;
        }

        let url = url.ok_or_else(|| "YouTube URL is required".to_string())?;

        Ok(Args {
            url,
            prompt,
            model,
            api_key,
            config_path,
            verbose,
        })
    }

    fn usage() -> String {
        r#"Usage: youtube-summary <URL> [OPTIONS]

Arguments:
  <URL>                    YouTube video URL

Options:
  -p, --prompt <PROMPT>    Custom prompt for the summary
  -m, --model <MODEL>      Model to use: haiku, sonnet, opus (default: sonnet)
  -k, --api-key <KEY>      Anthropic API key (overrides env/config)
  -c, --config <PATH>      Path to config file
  -v, --verbose            Show verbose output
  -h, --help               Show this help message

Examples:
  youtube-summary "https://youtube.com/watch?v=VIDEO_ID"
  youtube-summary "https://youtube.com/watch?v=VIDEO_ID" -p "Is this worth watching?"
  youtube-summary "https://youtube.com/watch?v=VIDEO_ID" -m opus -v"#
            .to_string()
    }
}
