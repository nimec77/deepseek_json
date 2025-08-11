use anyhow::{Context, Result};
use std::env;

const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";
const DEFAULT_MODEL: &str = "deepseek-chat";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_TIMEOUT: u64 = 180;

/// Configuration structure for the DeepSeek client
#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();

        let api_key = env::var("DEEPSEEK_API_KEY")
            .context("DEEPSEEK_API_KEY environment variable not set")?;

        let base_url =
            env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        let max_tokens = env::var("DEEPSEEK_MAX_TOKENS")
            .unwrap_or_else(|_| DEFAULT_MAX_TOKENS.to_string())
            .parse::<u32>()
            .context("MAX_TOKENS must be a valid number")?;

        let temperature = env::var("DEEPSEEK_TEMPERATURE")
            .unwrap_or_else(|_| DEFAULT_TEMPERATURE.to_string())
            .parse::<f32>()
            .context("TEMPERATURE must be a valid number")?;

        let timeout = env::var("DEEPSEEK_TIMEOUT")
            .unwrap_or_else(|_| DEFAULT_TIMEOUT.to_string())
            .parse::<u64>()
            .context("TIMEOUT must be a valid number")?;

        Ok(Self {
            api_key,
            base_url,
            model,
            max_tokens,
            temperature,
            timeout,
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            anyhow::bail!("API key cannot be empty");
        }

        if self.temperature < 0.0 || self.temperature > 2.0 {
            anyhow::bail!("Temperature must be between 0.0 and 2.0");
        }

        if self.max_tokens == 0 {
            anyhow::bail!("Max tokens must be greater than 0");
        }

        if self.timeout == 0 {
            anyhow::bail!("Timeout must be greater than 0");
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}
