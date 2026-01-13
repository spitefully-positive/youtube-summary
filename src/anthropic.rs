use crate::config::Config;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

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
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ApiError,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

pub async fn summarize(config: &Config, transcript: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let user_content = format!("{}\n\n---\n\nTranscript:\n{}", config.prompt, transcript);

    let request = Request {
        model: config.model.api_name().to_string(),
        max_tokens: 4096,
        messages: vec![Message {
            role: "user".to_string(),
            content: user_content,
        }],
    };

    if config.verbose {
        eprintln!("[verbose] Model: {}", config.model.api_name());
        eprintln!("[verbose] Transcript length: {} chars", transcript.len());
        eprintln!("[verbose] Sending request to Anthropic API...");
    }

    let response = client
        .post(API_URL)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", API_VERSION)
        .header("content-type", "application/json")
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
        .content
        .into_iter()
        .map(|block| block.text)
        .collect::<Vec<_>>()
        .join("\n");

    if config.verbose {
        eprintln!("[verbose] Response received: {} chars", text.len());
    }

    Ok(text)
}
