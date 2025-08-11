use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::env;

use deepseek_json::{init, run, App, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }

    // Parse command line arguments
    let matches = Command::new("deepseek-json")
        .version("0.1.0")
        .author("Your Name <nimec77@gmail.com>")
        .about("A CLI tool for interacting with DeepSeek API and getting structured JSON responses")
        .arg(
            Arg::new("query")
                .short('q')
                .long("query")
                .value_name("QUERY")
                .help("Send a single query and exit (non-interactive mode)")
                .required(false),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL")
                .help("Override the default model")
                .required(false),
        )
        .arg(
            Arg::new("temperature")
                .short('t')
                .long("temperature")
                .value_name("TEMPERATURE")
                .help("Set the temperature for response generation (0.0-2.0)")
                .required(false),
        )
        .arg(
            Arg::new("max-tokens")
                .long("max-tokens")
                .value_name("MAX_TOKENS")
                .help("Set the maximum number of tokens in the response")
                .required(false),
        )
        .get_matches();

    // Handle single query mode
    if let Some(query) = matches.get_one::<String>("query") {
        return handle_single_query(query, &matches).await;
    }

    // Run in interactive mode
    run().await.context("Failed to run application")
}

/// Handle a single query in non-interactive mode
async fn handle_single_query(query: &str, matches: &clap::ArgMatches) -> Result<()> {
    // Create custom configuration if needed
    let app = if has_custom_config(matches) {
        let mut config = Config::load().context("Failed to load configuration")?;

        if let Some(model) = matches.get_one::<String>("model") {
            config.model = model.clone();
        }

        if let Some(temp_str) = matches.get_one::<String>("temperature") {
            config.temperature = temp_str.parse().context("Invalid temperature value")?;
        }

        if let Some(tokens_str) = matches.get_one::<String>("max-tokens") {
            config.max_tokens = tokens_str.parse().context("Invalid max-tokens value")?;
        }

        App::with_config(config)?
    } else {
        init()?
    };

    // Send the request
    let response = app
        .send_request(query)
        .await
        .context("Failed to process query")?;

    // Display the response in a clean format
    println!(
        "{}",
        serde_json::to_string_pretty(&response).context("Failed to serialize response")?
    );

    Ok(())
}

/// Check if any custom configuration options were provided
fn has_custom_config(matches: &clap::ArgMatches) -> bool {
    matches.get_one::<String>("model").is_some()
        || matches.get_one::<String>("temperature").is_some()
        || matches.get_one::<String>("max-tokens").is_some()
}
