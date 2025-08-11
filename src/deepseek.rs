use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Define the expected JSON response structure from DeepSeek
#[derive(Debug, Serialize, Deserialize)]
pub struct DeepSeekResponse {
    pub title: String,
    pub description: String,
    pub content: String,
    pub category: Option<String>,
    pub timestamp: Option<String>,
    pub confidence: Option<f32>,
}

/// API request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    response_format: ResponseFormat,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

/// DeepSeek API client
#[derive(Clone)]
pub struct DeepSeekClient {
    client: Client,
    config: Config,
}

impl DeepSeekClient {
/// Create a new DeepSeek client with the given configuration
pub fn new(config: Config) -> Result<Self> {
    config.validate()?;

    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .user_agent("openai_chat/0.1.0")
        .build()
        .context("Failed to create HTTP client")?;

    Ok(Self { client, config })
}
    /// Send a request to the DeepSeek API and return a structured response
    pub async fn send_request(&self, user_input: &str) -> Result<DeepSeekResponse> {
        let current_timestamp = Utc::now().to_rfc3339();
        
        let json_format_prompt = format!(r#"
Please respond with a JSON object containing the following fields:
{{
  "title": "A concise title for the topic (string)",
  "description": "A brief description or summary (string)",
  "content": "The main content or detailed response (string)",
  "category": "Optional category classification (string or null)",
  "timestamp": "Current response timestamp: {} (string)",
  "confidence": "Optional confidence score between 0.0 and 1.0 (number or null)"
}}

Make sure to provide valid JSON format in your response. Use the provided timestamp as the current response time."#, current_timestamp);

        let combined_prompt = format!("{}\n\n{}", user_input, json_format_prompt);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant that always responds with valid JSON in the specified format.".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: combined_prompt,
                },
            ],
            response_format: ResponseFormat {
                format_type: "json_object".to_string(),
            },
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("API request failed: {}", error_text));
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        let content = &api_response.choices[0].message.content;
        let parsed_response: DeepSeekResponse = serde_json::from_str(content)
            .context("Failed to parse JSON response from DeepSeek")?;

        Ok(parsed_response)
    }
}
