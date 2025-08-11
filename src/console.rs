use anyhow::{Context, Result};
use colored::*;
use std::io::{self, Write};

use crate::deepseek::{DeepSeekClient, DeepSeekError, DeepSeekResponse};

/// Console interface for the DeepSeek application
pub struct Console {
    client: DeepSeekClient,
}

impl Console {
    /// Create a new console interface with the provided DeepSeek client
    pub fn new(client: DeepSeekClient) -> Self {
        Self { client }
    }

    /// Display the welcome message and application information
    pub fn display_welcome() {
        println!(
            "{}",
            "🤖 DeepSeek JSON Chat Application".bright_blue().bold()
        );
        println!("{}", "This application sends your queries to DeepSeek and returns structured JSON responses.".blue());
        println!(
            "{}",
            "Make sure to set DEEPSEEK_API_KEY environment variable.".blue()
        );
        println!("{}", "Type '/quit' or '/exit' to stop.\n".blue());
    }

    /// Get user input from the console
    pub fn get_user_input() -> Result<String> {
        print!("{}", "💬 Enter your question: ".bright_cyan().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;

        Ok(input.trim().to_string())
    }

    /// Check if the input is a quit command
    pub fn is_quit_command(input: &str) -> bool {
        input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit")
    }

    /// Display a loading message
    pub fn display_loading() {
        println!("{}", "🔄 Sending request to DeepSeek...".blue().italic());
    }

    /// Display the structured response from DeepSeek
    pub fn display_response(response: &DeepSeekResponse) {
        println!("\n{}", "📋 Structured Response:".bright_green().bold());
        println!(
            "{}",
            "┌─────────────────────────────────────────────────────────────".green()
        );
        println!(
            "{} {}",
            "│ 🏷️  Title:".green(),
            response.title.bright_white().bold()
        );
        println!(
            "{} {}",
            "│ 📝 Description:".green(),
            response.description.white()
        );
        println!("{} {}", "│ 📄 Content:".green(), response.content.white());

        if let Some(category) = &response.category {
            println!("{} {}", "│ 🏪 Category:".green(), category.white());
        }

        if let Some(timestamp) = &response.timestamp {
            println!("{} {}", "│ ⏰ Timestamp:".green(), timestamp.white());
        }

        if let Some(confidence) = response.confidence {
            println!(
                "{} {}",
                "│ 🎯 Confidence:".green(),
                format!("{:.2}", confidence).white()
            );
        }

        println!(
            "{}",
            "└─────────────────────────────────────────────────────────────\n".green()
        );
    }

    /// Display an error message with context-aware messaging
    pub fn display_error(error: &anyhow::Error) {
        // Try to downcast to our custom DeepSeekError
        if let Some(deepseek_error) = error.downcast_ref::<DeepSeekError>() {
            Self::display_deepseek_error(deepseek_error);
        } else {
            // Fallback for other errors
            println!(
                "{} {}",
                "❌ Error:".bright_red().bold(),
                error.to_string().red()
            );
            println!(
                "{}",
                "Please check your configuration and try again.\n".red()
            );
        }
    }

    /// Display a DeepSeekError with appropriate styling and context
    pub fn display_deepseek_error(error: &DeepSeekError) {
        let user_message = error.user_message();

        match error {
            DeepSeekError::ServerBusy => {
                println!("{}", user_message.bright_yellow().bold());
                println!(
                    "{}",
                    "💡 Tip: Try again in a few minutes when server load is lower.".yellow()
                );
            }
            DeepSeekError::NetworkError { .. } => {
                println!("{}", user_message.bright_red().bold());
                println!(
                    "{}",
                    "💡 Tip: Check your internet connection and firewall settings.".red()
                );
            }
            DeepSeekError::Timeout { .. } => {
                println!("{}", user_message.bright_yellow().bold());
                println!(
                    "{}",
                    "💡 Tip: The server might be overloaded. Try again later.".yellow()
                );
            }
            DeepSeekError::ApiError { status, .. } => {
                println!("{}", user_message.bright_red().bold());
                match *status {
                    401 => println!(
                        "{}",
                        "💡 Tip: Check your DEEPSEEK_API_KEY environment variable.".red()
                    ),
                    403 => println!(
                        "{}",
                        "💡 Tip: Your API key may not have sufficient permissions.".red()
                    ),
                    429 => println!(
                        "{}",
                        "💡 Tip: You've hit the rate limit. Wait before trying again.".red()
                    ),
                    _ => println!(
                        "{}",
                        "💡 Tip: Check the DeepSeek API documentation for more details.".red()
                    ),
                }
            }
            DeepSeekError::ParseError { .. } => {
                println!("{}", user_message.bright_magenta().bold());
                println!(
                    "{}",
                    "💡 Tip: The server response was unexpected. Try rephrasing your query."
                        .magenta()
                );
            }
            DeepSeekError::ConfigError { .. } => {
                println!("{}", user_message.bright_red().bold());
                println!(
                    "{}",
                    "💡 Tip: Check your environment variables and configuration.".red()
                );
            }
        }
        println!(); // Add spacing
    }

    /// Display a goodbye message
    pub fn display_goodbye() {
        println!("{}", "👋 Goodbye!".bright_yellow().bold());
    }

    /// Run the main console loop
    pub async fn run(&self) -> Result<()> {
        Self::display_welcome();

        loop {
            let input = Self::get_user_input()?;

            if input.is_empty() {
                continue;
            }

            if Self::is_quit_command(&input) {
                Self::display_goodbye();
                break;
            }

            Self::display_loading();

            match self.client.send_request(&input).await {
                Ok(response) => Self::display_response(&response),
                Err(e) => Self::display_deepseek_error(&e),
            }
        }

        Ok(())
    }
}
