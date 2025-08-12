use anyhow::{Context, Result};

pub mod config;
pub mod console;
pub mod deepseek;
pub mod taskfinisher;

pub use config::Config;
pub use console::Console;
pub use deepseek::{DeepSeekClient, DeepSeekError, DeepSeekResponse};
pub use taskfinisher::{
    parse_taskfinisher_response, build_system_prompt, AnswersPayload, TaskFinisherResult,
    DEFAULT_MAX_QUESTIONS,
};

/// Application struct that encapsulates the core functionality
pub struct App {
    client: DeepSeekClient,
    console: Console,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        // Load configuration
        let config = Config::load().context("Failed to load configuration")?;

        // Initialize DeepSeek client
        let client = DeepSeekClient::new(config).map_err(|e| anyhow::anyhow!("{}", e))?;

        // Create console interface
        let console = Console::new(client.clone());

        Ok(Self { client, console })
    }

    /// Create a new application instance with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        // Initialize DeepSeek client
        let client = DeepSeekClient::new(config).map_err(|e| anyhow::anyhow!("{}", e))?;

        // Create console interface
        let console = Console::new(client.clone());

        Ok(Self { client, console })
    }

    /// Run the application
    pub async fn run(&self) -> Result<()> {
        self.console
            .run()
            .await
            .context("Application execution failed")
    }

    /// Run TaskFinisher-JSON interactive flow. If `initial_prompt` is None, the user will be asked.
    pub async fn run_taskfinisher(&self, initial_prompt: Option<&str>, max_questions: u32) -> Result<()> {
        self.console
            .run_taskfinisher(initial_prompt, max_questions)
            .await
            .context("TaskFinisher flow failed")
    }

    /// Get a reference to the DeepSeek client
    pub fn client(&self) -> &DeepSeekClient {
        &self.client
    }

    /// Send a single request and return the response (useful for non-interactive usage)
    pub async fn send_request(&self, input: &str) -> Result<DeepSeekResponse, DeepSeekError> {
        self.client.send_request(input).await
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create default application")
    }
}

/// Initialize the application with environment setup
pub fn init() -> Result<App> {
    App::new()
}

/// Run the application with default settings
pub async fn run() -> Result<()> {
    let app = init()?;
    app.run().await
}
