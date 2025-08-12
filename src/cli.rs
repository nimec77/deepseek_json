use anyhow::{Context, Result};
use clap::Parser;
use std::env;

use crate::{App, Config, DEFAULT_MAX_QUESTIONS};

/// Command line interface for the application
#[derive(Parser, Debug)]
#[command(name = "deepseek-json")]
#[command(version = "0.1.0")]
#[command(author = "Your Name <nimec77@gmail.com>")]
#[command(
    about = "A CLI tool for interacting with DeepSeek API and getting structured JSON responses"
)]
pub struct Cli {
    /// Send a single query and exit (non-interactive mode)
    #[arg(short, long)]
    pub query: Option<String>,

    /// Override the default model
    #[arg(short, long, default_value = "deepseek-chat")]
    pub model: String,

    /// Set the temperature for response generation (0.0-2.0)
    #[arg(short, long, default_value_t = 0.7)]
    pub temperature: f32,

    /// Set the maximum number of tokens in the response
    #[arg(long, default_value_t = 4096)]
    pub max_tokens: u32,

    /// Request timeout in seconds
    #[arg(long, default_value_t = 180)]
    pub timeout: u64,

    /// DeepSeek API base URL
    #[arg(long)]
    pub base_url: Option<String>,

    /// Enable TaskFinisher-JSON mode
    #[arg(long, default_value_t = false)]
    pub taskfinisher: bool,

    /// Maximum clarifying questions for TaskFinisher-JSON mode
    #[arg(long, default_value_t = DEFAULT_MAX_QUESTIONS)]
    pub max_questions: u32,
}

/// Entry point for running the application via CLI
pub async fn run_cli() -> Result<()> {
    // Initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load environment variables once at startup
    dotenv::dotenv().ok();

    // Parse command line arguments
    let cli = Cli::parse();

    // Handle single query mode / taskfinisher mode / interactive
    if cli.taskfinisher {
        return handle_taskfinisher_mode(&cli).await;
    }
    if let Some(query) = &cli.query {
        return handle_single_query(query, &cli).await;
    }

    // Run in interactive mode
    crate::run().await.context("Failed to run application")
}

/// Handle a single query in non-interactive mode
async fn handle_single_query(query: &str, cli: &Cli) -> Result<()> {
    // Create configuration with CLI overrides
    let mut config = Config::load().context("Failed to load configuration")?;

    // Apply CLI overrides
    config.model = cli.model.clone();
    config.temperature = cli.temperature;
    config.max_tokens = cli.max_tokens;
    config.timeout = cli.timeout;

    if let Some(base_url) = &cli.base_url {
        config.base_url = base_url.clone();
    }

    let app = App::with_config(config)?;

    // Send the request
    let response = app
        .send_request(query)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to process query: {}", e))?;

    // Display the response in a clean format
    println!(
        "{}",
        serde_json::to_string_pretty(&response).context("Failed to serialize response")?
    );

    Ok(())
}

/// Handle TaskFinisher-JSON mode
async fn handle_taskfinisher_mode(cli: &Cli) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration")?;
    config.model = cli.model.clone();
    config.temperature = cli.temperature;
    config.max_tokens = cli.max_tokens;
    config.timeout = cli.timeout;
    if let Some(base_url) = &cli.base_url {
        config.base_url = base_url.clone();
    }

    let app = App::with_config(config)?;

    let initial_prompt = cli.query.as_deref();
    app.run_taskfinisher(initial_prompt, cli.max_questions)
        .await
}
