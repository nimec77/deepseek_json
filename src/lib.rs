use anyhow::{Context, Result};

pub mod config;
pub mod deepseek;
pub mod console;

pub use config::Config;
pub use deepseek::{DeepSeekClient, DeepSeekResponse};
pub use console::Console;

/// Application struct that encapsulates the core functionality
pub struct App {
    client: DeepSeekClient,
    console: Console,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        // Load configuration
        let config = Config::load()
            .context("Failed to load configuration")?;

        // Initialize DeepSeek client
        let client = DeepSeekClient::new(config)
            .context("Failed to initialize DeepSeek client")?;

        // Create console interface
        let console = Console::new(client.clone());

        Ok(Self { client, console })
    }

    /// Create a new application instance with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        // Initialize DeepSeek client
        let client = DeepSeekClient::new(config)
            .context("Failed to initialize DeepSeek client")?;

        // Create console interface  
        let console = Console::new(client.clone());

        Ok(Self { client, console })
    }

    /// Run the application
    pub async fn run(&self) -> Result<()> {
        self.console.run().await
            .context("Application execution failed")
    }

    /// Get a reference to the DeepSeek client
    pub fn client(&self) -> &DeepSeekClient {
        &self.client
    }

    /// Send a single request and return the response (useful for non-interactive usage)
    pub async fn send_request(&self, input: &str) -> Result<DeepSeekResponse> {
        self.client.send_request(input).await
            .context("Failed to send request")
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create default application")
    }
}

/// Initialize the application with environment setup
pub fn init() -> Result<App> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    App::new()
}

/// Run the application with default settings
pub async fn run() -> Result<()> {
    let app = init()?;
    app.run().await
}
