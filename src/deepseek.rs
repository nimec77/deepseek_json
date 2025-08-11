use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;

/// Custom error types for DeepSeek API interactions
#[derive(Error, Debug)]
pub enum DeepSeekError {
    #[error("DeepSeek servers are currently busy. Please try again in a few moments.")]
    ServerBusy,
    
    #[error("Network connection failed: {message}")]
    NetworkError { message: String },
    
    #[error("Request timed out after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },
    
    #[error("Failed to parse response: {message}")]
    ParseError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

impl DeepSeekError {
    /// Check if the error indicates server is busy
    pub fn is_server_busy(&self) -> bool {
        matches!(self, DeepSeekError::ServerBusy)
    }
    
    /// Check if the error is a network-related issue
    pub fn is_network_error(&self) -> bool {
        matches!(self, DeepSeekError::NetworkError { .. })
    }
    
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            DeepSeekError::ServerBusy => {
                "🚫 DeepSeek servers are currently busy. Please try again in a few moments.".to_string()
            }
            DeepSeekError::NetworkError { .. } => {
                "🌐 Network connection failed. Please check your internet connection and try again.".to_string()
            }
            DeepSeekError::Timeout { seconds } => {
                format!("⏰ Request timed out after {} seconds. The server might be overloaded.", seconds)
            }
            DeepSeekError::ApiError { status, .. } => {
                match *status {
                    429 => "🚫 Rate limit exceeded. Please wait a moment before trying again.".to_string(),
                    503 => "🚫 Service temporarily unavailable. Please try again later.".to_string(),
                    502 | 504 => "🚫 Server gateway error. Please try again in a few moments.".to_string(),
                    _ => format!("❌ API error ({}). Please try again later.", status),
                }
            }
            DeepSeekError::ParseError { .. } => {
                "⚠️ Failed to parse server response. Please try again.".to_string()
            }
            DeepSeekError::ConfigError { message } => {
                format!("⚙️ Configuration error: {}", message)
            }
        }
    }
}

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
    pub fn new(config: Config) -> Result<Self, DeepSeekError> {
        config.validate().map_err(|e| DeepSeekError::ConfigError {
            message: e.to_string(),
        })?;

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .user_agent("deepseek_json/0.1.0")
            .build()
            .map_err(|e| DeepSeekError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self { client, config })
    }
    /// Send a request to the DeepSeek API and return a structured response
    pub async fn send_request(&self, user_input: &str) -> Result<DeepSeekResponse, DeepSeekError> {
        let current_timestamp = Utc::now().to_rfc3339();

        let json_format_prompt = format!(
            r#"
                Please respond with a JSON object containing the following fields:
                {{
                "title": "A concise title for the topic (string)",
                "description": "A brief description or summary (string)",
                "content": "The main content or detailed response (string)",
                "category": "Optional category classification (string or null)",
                "timestamp": "Current response timestamp: {} (string)",
                "confidence": "Optional confidence score between 0.0 and 1.0 (number or null)"
                }}

                Make sure to provide valid JSON format in your response. Use the provided timestamp as the current response time.
                Do not include any other text or comments in your response.
            "#,
            current_timestamp
        );

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

        // First, do a quick health check with a shorter timeout
        self.check_server_availability().await?;

        // Send the actual request
        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        // Handle HTTP status codes
        let status = response.status();
        if !status.is_success() {
            return Err(self.handle_error_response(status, response).await);
        }

        // Parse the response
        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| DeepSeekError::ParseError {
                message: format!("Failed to parse API response: {}", e),
            })?;

        if api_response.choices.is_empty() {
            return Err(DeepSeekError::ParseError {
                message: "No choices in API response".to_string(),
            });
        }

        let content = &api_response.choices[0].message.content;
        let parsed_response: DeepSeekResponse = serde_json::from_str(content)
            .map_err(|e| DeepSeekError::ParseError {
                message: format!("Failed to parse JSON response from DeepSeek: {}", e),
            })?;

        Ok(parsed_response)
    }

    /// Check if the server is available with a quick health check
    async fn check_server_availability(&self) -> Result<(), DeepSeekError> {
        let health_client = Client::builder()
            .timeout(Duration::from_secs(10)) // Short timeout for health check
            .user_agent("deepseek_json/0.1.0")
            .build()
            .map_err(|e| DeepSeekError::NetworkError {
                message: format!("Failed to create health check client: {}", e),
            })?;

        let health_check = health_client
            .get(&self.config.base_url)
            .send()
            .await
            .map_err(|e| self.map_reqwest_error(e))?;

        let status = health_check.status();
        
        // Check for server busy conditions
        match status {
            StatusCode::TOO_MANY_REQUESTS => Err(DeepSeekError::ServerBusy),
            StatusCode::SERVICE_UNAVAILABLE => Err(DeepSeekError::ServerBusy),
            StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => Err(DeepSeekError::ServerBusy),
            _ => Ok(()),
        }
    }

    /// Map reqwest errors to our custom error types
    fn map_reqwest_error(&self, error: reqwest::Error) -> DeepSeekError {
        if error.is_timeout() {
            return DeepSeekError::Timeout {
                seconds: self.config.timeout,
            };
        }

        if error.is_connect() {
            return DeepSeekError::NetworkError {
                message: "Failed to connect to server".to_string(),
            };
        }

        if error.is_request() {
            return DeepSeekError::NetworkError {
                message: "Request failed".to_string(),
            };
        }

        // Check for specific network-related errors
        let error_msg = error.to_string().to_lowercase();
        if error_msg.contains("dns") {
            return DeepSeekError::NetworkError {
                message: "DNS resolution failed".to_string(),
            };
        }

        if error_msg.contains("connection refused") {
            return DeepSeekError::NetworkError {
                message: "Connection refused by server".to_string(),
            };
        }

        if error_msg.contains("network") || error_msg.contains("connection") {
            return DeepSeekError::NetworkError {
                message: error.to_string(),
            };
        }

        DeepSeekError::NetworkError {
            message: format!("Request error: {}", error),
        }
    }

    /// Handle error responses from the server
    async fn handle_error_response(&self, status: StatusCode, response: reqwest::Response) -> DeepSeekError {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        match status {
            StatusCode::TOO_MANY_REQUESTS => DeepSeekError::ServerBusy,
            StatusCode::SERVICE_UNAVAILABLE => DeepSeekError::ServerBusy,
            StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => DeepSeekError::ServerBusy,
            _ => DeepSeekError::ApiError {
                status: status.as_u16(),
                message: error_text,
            },
        }
    }
}
