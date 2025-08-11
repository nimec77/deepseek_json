# DeepSeek JSON Chat Application

A robust Rust CLI application that sends requests to DeepSeek's API and receives structured JSON responses. The application automatically adds JSON format specifications to user prompts and parses the structured responses for console display with beautiful formatting.

## Features

- 🤖 **DeepSeek Integration**: Direct integration with DeepSeek's chat API using `deepseek-chat` model
- 📋 **Structured JSON Responses**: Enforces JSON format with predefined schema using response_format
- 🎯 **Field Extraction**: Automatically parses and displays JSON fields with colored output
- 🛡️ **Advanced Error Handling**: Custom error types with user-friendly messages and recovery suggestions
- 🔄 **Interactive Mode**: Continuous chat interface until user exits with `/quit` or `/exit` commands
- 💻 **CLI Interface**: Command-line interface with single query mode support
- ⚙️ **Configurable**: Customizable model, temperature, token limits, and request timeouts
- 🎨 **Beautiful Output**: Colored and formatted console output with emojis for better readability
- ⏱️ **Timeout Support**: Configurable request timeouts with health checks
- 🌐 **Network Resilience**: Server availability checks and intelligent error handling
- 🔧 **Modular Architecture**: Clean separation of concerns with dedicated modules

## JSON Response Structure

The application requests responses in the following JSON format:

```json
{
  "title": "A concise title for the topic",
  "description": "A brief description or summary", 
  "content": "The main content or detailed response",
  "category": "Optional category classification",
  "timestamp": "Optional timestamp in ISO 8601 format",
  "confidence": "Optional confidence score between 0.0 and 1.0"
}
```

## Setup

### Quick Setup (Recommended)

1. **Clone and navigate to the project**:
   ```bash
   git clone <repository-url>
   cd deepseek_json
   ```

2. **Run the setup script**:
   ```bash
   chmod +x setup.sh
   ./setup.sh
   ```
   This script will:
   - Create the `.env` file from template
   - Build the project in release mode
   - Provide usage instructions

3. **Get your DeepSeek API key**:
   - Visit [DeepSeek Platform](https://platform.deepseek.com)
   - Create an account and generate an API key
   - Edit `.env` and replace `your_deepseek_api_key_here` with your actual API key

### Manual Setup

1. **Set up environment variables**:
   ```bash
   cp env.example .env
   # Edit .env and add your DeepSeek API key
   ```

2. **Build and run**:
   ```bash
   cargo build --release
   cargo run
   ```

## Usage

### Interactive Mode

1. **Run the application**: `cargo run`
2. **Enter your questions** when prompted
3. The application will:
   - Add JSON format instructions to your prompt
   - Send the request to DeepSeek
   - Parse the JSON response
   - Display structured fields in the console with colors
4. **Type `/quit` or `/exit`** to stop

### CLI Mode (Single Query)

Use command-line arguments for non-interactive usage:

```bash
# Basic single query
cargo run -- --query "Tell me about Rust programming"

# With custom model
cargo run -- --query "Explain async programming" --model "deepseek-chat"

# With custom temperature and max tokens
cargo run -- --query "Write a poem" --temperature 1.2 --max-tokens 500

# Short form options
cargo run -- -q "What is machine learning?" -t 0.8
```

### Command-Line Options

- `-q, --query <QUERY>`: Send a single query and exit (non-interactive mode)
- `-m, --model <MODEL>`: Override the default model (default: `deepseek-chat`)
- `-t, --temperature <TEMPERATURE>`: Set temperature for response generation (0.0-2.0, default: 0.7)
- `--max-tokens <MAX_TOKENS>`: Set maximum number of tokens in response (default: 4096)
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Example

### Interactive Mode
```
🤖 DeepSeek JSON Chat Application
This application sends your queries to DeepSeek and returns structured JSON responses.
Make sure to set DEEPSEEK_API_KEY environment variable.
Type '/quit' or '/exit' to stop.

💬 Enter your question: Tell me about Rust programming language

🔄 Sending request to DeepSeek...

📋 Structured Response:
┌─────────────────────────────────────────────────────────────
│ 🏷️  Title: Rust Programming Language Overview
│ 📝 Description: A systems programming language focused on safety and performance
│ 📄 Content: Rust is a modern systems programming language that focuses on safety, speed, and concurrency. It prevents segfaults and guarantees thread safety.
│ 🏪 Category: Programming Languages
│ ⏰ Timestamp: 2024-01-15T10:30:00Z
│ 🎯 Confidence: 0.95
└─────────────────────────────────────────────────────────────

💬 Enter your question: /quit
👋 Goodbye!
```

### CLI Mode
```bash
$ cargo run -- --query "What is machine learning?" --temperature 0.8
{
  "title": "Introduction to Machine Learning",
  "description": "An overview of machine learning concepts and applications",
  "content": "Machine learning is a subset of artificial intelligence that enables computers to learn and make decisions from data without being explicitly programmed...",
  "category": "Technology",
  "timestamp": "2024-01-15T10:35:00Z",
  "confidence": 0.92
}
```

## Dependencies

- `tokio`: Async runtime for handling HTTP requests and concurrent operations
- `reqwest`: HTTP client for API communication with JSON support
- `serde`: Serialization/deserialization framework with derive macros
- `serde_json`: JSON parsing and manipulation support
- `anyhow`: Simplified error handling and context management
- `thiserror`: Custom error type derivation for structured error handling
- `dotenv`: Environment variable management from `.env` files
- `clap`: Command-line argument parsing with derive macros
- `colored`: Terminal color output for beautiful console display
- `chrono`: Date and time handling with serialization support

## Configuration

The application can be configured using environment variables in your `.env` file:

### Required Configuration
- `DEEPSEEK_API_KEY`: Your DeepSeek API key (required)

### Optional Configuration
- `DEEPSEEK_BASE_URL`: API base URL (default: `https://api.deepseek.com`)
- `DEEPSEEK_MODEL`: Model to use (default: `deepseek-chat`)
- `DEEPSEEK_MAX_TOKENS`: Maximum tokens in response (default: `4096`)
- `DEEPSEEK_TEMPERATURE`: Response generation temperature 0.0-2.0 (default: `0.7`)
- `DEEPSEEK_TIMEOUT`: Request timeout in seconds (default: `180`)

### Example `.env` file:
```env
DEEPSEEK_API_KEY=your_actual_api_key_here
DEEPSEEK_BASE_URL=https://api.deepseek.com
DEEPSEEK_MODEL=deepseek-chat
DEEPSEEK_MAX_TOKENS=4096
DEEPSEEK_TEMPERATURE=0.7
DEEPSEEK_TIMEOUT=180
```

**Note**: The minimal `.env` file only requires `DEEPSEEK_API_KEY`. See `env.example` for the template.

**Note**: Command-line arguments will override environment variable settings.

## Project Architecture

The application is built with a modular architecture for maintainability and extensibility:

### Core Modules

- **`config.rs`**: Configuration management and environment variable handling
  - Validates configuration parameters
  - Provides sensible defaults
  - Supports both environment variables and command-line overrides

- **`deepseek.rs`**: DeepSeek API client and communication layer
  - Custom error types with `thiserror` integration
  - HTTP client with timeout and retry logic
  - JSON response parsing and validation
  - Server health checks and availability monitoring

- **`console.rs`**: User interface and terminal interaction
  - Colored output with emoji indicators
  - Interactive prompt handling
  - Error display with contextual help
  - Welcome and goodbye messages

- **`lib.rs`**: Application orchestration and public API
  - Main `App` struct that coordinates all components
  - Initialization and configuration loading
  - Both interactive and single-query modes

- **`main.rs`**: CLI argument parsing and entry point
  - Command-line argument processing with `clap`
  - Single query mode handling
  - Application startup and error handling

### Design Principles

- **Async-First**: Built on `tokio` for efficient I/O handling
- **Error Transparency**: Custom error types provide clear user feedback
- **Configuration Flexibility**: Multiple ways to configure the application
- **Separation of Concerns**: Each module has a single, well-defined responsibility
- **User Experience**: Prioritizes clear feedback and beautiful terminal output

## Error Handling

The application features advanced error handling with custom error types and user-friendly messaging:

### Error Types
- **ServerBusy**: Handles rate limiting and server overload scenarios
- **NetworkError**: DNS failures, connection issues, and network timeouts
- **Timeout**: Request timeouts with configurable duration
- **ApiError**: HTTP status code errors with context-aware messages
- **ParseError**: JSON parsing and response format issues
- **ConfigError**: Configuration validation and setup problems

### Error Features
- 🎯 **Context-Aware Messages**: Different error types show appropriate user guidance
- 💡 **Recovery Suggestions**: Each error type includes helpful tips for resolution
- 🎨 **Color-Coded Display**: Errors are displayed with appropriate colors and emojis
- 🔍 **Detailed Logging**: Comprehensive error context for troubleshooting
- 🛡️ **Graceful Degradation**: Application continues running after recoverable errors

### Health Checks
- Server availability checks before sending requests
- Automatic retry suggestions for transient failures
- Network connectivity validation
