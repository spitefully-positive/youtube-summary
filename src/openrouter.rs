use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{Error, Result};

const API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MODELS_URL: &str = "https://openrouter.ai/api/v1/models";

pub const DEFAULT_MODEL: &str = "anthropic/claude-haiku-4.5";

#[derive(Serialize)]
struct Request {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ApiError,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    id: String,
    name: String,
    context_length: Option<u64>,
    pricing: Option<Pricing>,
}

#[derive(Deserialize)]
struct Pricing {
    prompt: String,
    completion: String,
}

pub async fn summarize(config: &Config, transcript: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let user_content = format!("{}\n\n---\n\nTranscript:\n{}", config.prompt, transcript);

    let request = Request {
        model: config.model.clone(),
        max_tokens: 4096,
        messages: vec![Message {
            role: "user".to_string(),
            content: user_content,
        }],
    };

    if config.verbose {
        eprintln!("[verbose] Model: {}", config.model);
        eprintln!("[verbose] Transcript length: {} chars", transcript.len());
        eprintln!("[verbose] Sending request to OpenRouter API...");
    }

    let response = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| Error::ApiRequest(format!("Failed to send request: {}", e)))?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();

        // Try to parse error response
        if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
            return Err(Error::ApiRequest(format!(
                "API error ({}): {}",
                status, error_response.error.message
            )));
        }

        return Err(Error::ApiRequest(format!(
            "API error ({}): {}",
            status, error_text
        )));
    }

    let response: Response = response
        .json()
        .await
        .map_err(|e| Error::ApiRequest(format!("Failed to parse response: {}", e)))?;

    let text = response
        .choices
        .into_iter()
        .map(|choice| choice.message.content)
        .collect::<Vec<_>>()
        .join("\n");

    if config.verbose {
        eprintln!("[verbose] Response received: {} chars", text.len());
    }

    Ok(text)
}

pub async fn list_models(api_key: &str, search: Option<&str>, verbose: bool) -> Result<()> {
    let client = reqwest::Client::new();

    if verbose {
        eprintln!("[verbose] Fetching models from OpenRouter API...");
    }

    let response = client
        .get(MODELS_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| Error::ApiRequest(format!("Failed to fetch models: {}", e)))?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(Error::ApiRequest(format!(
            "API error ({}): {}",
            status, error_text
        )));
    }

    let models_response: ModelsResponse = response
        .json()
        .await
        .map_err(|e| Error::ApiRequest(format!("Failed to parse models response: {}", e)))?;

    // Filter models if search term provided
    let models: Vec<&ModelInfo> = models_response
        .data
        .iter()
        .filter(|m| {
            if let Some(term) = search {
                let term_lower = term.to_lowercase();
                m.id.to_lowercase().contains(&term_lower)
                    || m.name.to_lowercase().contains(&term_lower)
            } else {
                true
            }
        })
        .collect();

    if models.is_empty() {
        if let Some(term) = search {
            println!("No models found matching '{}'", term);
        } else {
            println!("No models found");
        }
        return Ok(());
    }

    // Print header
    println!(
        "{:<45} {:<40} {:>8}   PRICING (per 1M tokens)",
        "MODEL ID", "NAME", "CONTEXT"
    );
    println!("{}", "-".repeat(120));

    // Print models
    for model in &models {
        let context = model
            .context_length
            .map(format_context)
            .unwrap_or_else(|| "N/A".to_string());

        let pricing = model
            .pricing
            .as_ref()
            .map(format_pricing)
            .unwrap_or_else(|| "N/A".to_string());

        // Truncate long names
        let id = truncate(&model.id, 44);
        let name = truncate(&model.name, 39);

        println!("{:<45} {:<40} {:>8}   {}", id, name, context, pricing);
    }

    if verbose {
        eprintln!("\n[verbose] Total models displayed: {}", models.len());
    }

    Ok(())
}

fn format_context(context_length: u64) -> String {
    if context_length >= 1_000_000 {
        format!("{}M", context_length / 1_000_000)
    } else if context_length >= 1_000 {
        format!("{}k", context_length / 1_000)
    } else {
        format!("{}", context_length)
    }
}

fn format_pricing(pricing: &Pricing) -> String {
    // Parse the pricing strings (they're in dollars per token)
    // Convert to per million tokens for readability
    let prompt_per_million = parse_price(&pricing.prompt);
    let completion_per_million = parse_price(&pricing.completion);

    match (prompt_per_million, completion_per_million) {
        (Some(p), Some(c)) => format!("${:.2} / ${:.2}", p, c),
        _ => "N/A".to_string(),
    }
}

fn parse_price(price_str: &str) -> Option<f64> {
    let price: f64 = price_str.parse().ok()?;
    if price < 0.0 {
        return None; // Free or special pricing
    }
    // Price is per token, convert to per million
    Some(price * 1_000_000.0)
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
