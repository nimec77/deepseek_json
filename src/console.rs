use anyhow::{Context, Result};
use colored::*;
use std::io::{self, Write};

use crate::deepseek::{DeepSeekClient, DeepSeekResponse};

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
        println!("{}", "🤖 DeepSeek JSON Chat Application".bright_blue().bold());
        println!("{}", "This application sends your queries to DeepSeek and returns structured JSON responses.".blue());
        println!("{}", "Make sure to set DEEPSEEK_API_KEY environment variable.".blue());
        println!("{}", "Type '/quit' or '/exit' to stop.\n".blue());
    }

    /// Get user input from the console
    pub fn get_user_input() -> Result<String> {
        print!("{}", "💬 Enter your question: ".bright_cyan().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)
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
        println!("{}", "┌─────────────────────────────────────────────────────────────".green());
        println!("{} {}", "│ 🏷️  Title:".green(), response.title.bright_white().bold());
        println!("{} {}", "│ 📝 Description:".green(), response.description.white());
        println!("{} {}", "│ 📄 Content:".green(), response.content.white());
        
        if let Some(category) = &response.category {
            println!("{} {}", "│ 🏪 Category:".green(), category.white());
        }
        
        if let Some(timestamp) = &response.timestamp {
            println!("{} {}", "│ ⏰ Timestamp:".green(), timestamp.white());
        }
        
        if let Some(confidence) = response.confidence {
            println!("{} {}", "│ 🎯 Confidence:".green(), format!("{:.2}", confidence).white());
        }
        
        println!("{}", "└─────────────────────────────────────────────────────────────\n".green());
    }

    /// Display an error message
    pub fn display_error(error: &anyhow::Error) {
        println!("{} {}", "❌ Error:".bright_red().bold(), error.to_string().red());
        println!("{}", "Please check your API key and network connection.\n".red());
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
                Err(e) => Self::display_error(&e),
            }
        }

        Ok(())
    }
}
