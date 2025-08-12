# DeepSeek JSON Chat Application

A robust Rust CLI that sends requests to DeepSeek's API and receives structured JSON responses. The app injects strict JSON format instructions into your prompt and parses the model’s response for a clean, colored console display.

## Features

- 🤖 **DeepSeek integration**: Uses the `deepseek-chat` model via `/chat/completions` with `response_format = json_object`.
- 📋 **Structured JSON responses**: Enforces a stable schema and validates/parses the assistant output.
- 🧩 **TaskFinisher-JSON mode**: Interactive clarifications flow that yields a final technical task JSON artifact with a self-stop token.
- 🎯 **Field extraction**: Nicely formatted, colored rendering of JSON fields in the terminal.
- 🛡️ **Advanced error handling**: Clear, user-friendly messages for server busy, timeouts, API and parsing errors.
- 🔄 **Interactive mode**: Continuous chat until you exit with `/quit` or `/exit`.
- 💻 **CLI mode**: Send a single query and print the JSON result.
- ⚙️ **Configurable**: Model, temperature, token limits, base URL, and request timeout.
- 🎨 **Beautiful output**: Colored, emoji-enhanced display.
- ⏱️ **Timeouts + retries**: Configurable timeouts and automatic exponential backoff on transient errors (3 attempts).
- 🌐 **Network resilience**: Smart retry conditions for rate limits and network issues; graceful error messages. No pre-flight health checks are performed.
- 🔧 **Modular architecture**: Clean separation of config, client, console UI, and TaskFinisher logic.
- ⚡ **Signal handling**: Ctrl+C exits gracefully. If pressed during a request, the request is canceled and the app exits.

## JSON response structure

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

### Interactive mode

1. **Run the application**: `cargo run`
2. **Enter your questions** when prompted
3. The application will:
   - Add JSON format instructions to your prompt
   - Send the request to DeepSeek
   - Parse the JSON response
   - Display structured fields in the console with colors
4. **Exit options**:
   - Type `/quit` or `/exit` to stop gracefully
   - Press `Ctrl+C` at any time to exit (if pressed during a request, it cancels the request and exits)

### CLI mode (single query)

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

# With custom base URL and timeout
cargo run -- -q "Explain quantum computing" --base-url "https://custom-api.example.com" --timeout 300

# TaskFinisher-JSON mode (technical task artifact)
cargo run -- --taskfinisher --query "Build a Rust service that fetches prices and caches them" --max-questions 3
```

### TaskFinisher-JSON mode

When run with `--taskfinisher`, the app enters a clarifications flow to produce a final technical task artifact:

- The assistant may ask up to `--max-questions N` clarifying questions (default: 3). Internally, there is a hard cap of 5 rounds.
- You answer questions one-by-one interactively:
  - Press Enter to skip a question.
  - Type `/proceed` to finalize early.
  - Type `/quit` or `/exit` to abort.
- The final artifact includes `"status":"final"` and `"end_token":"【END】"` and then stops.
- You can seed the very first message with `--query "..."`; otherwise you will be prompted for it.

### Command-line options

- `-q, --query <QUERY>`: Send a single query and exit (non-interactive mode)
- `-m, --model <MODEL>`: Override the default model (default: `deepseek-chat`)
- `-t, --temperature <TEMPERATURE>`: Set temperature for response generation (0.0-2.0, default: 0.7)
- `--max-tokens <MAX_TOKENS>`: Set maximum number of tokens in response (default: 4096)
- `--timeout <TIMEOUT>`: Request timeout in seconds (default: 180)
- `--base-url <BASE_URL>`: DeepSeek API base URL (overrides environment variable)
- `--taskfinisher`: Enable TaskFinisher-JSON mode
- `--max-questions <N>`: Limit clarifying questions in TaskFinisher mode (default: 3)
- `-h, --help`: Show help information
- `-V, --version`: Show version information

Notes:
- CLI arguments override environment variables.
- `.env` is loaded once at startup.

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

### CLI mode
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
- `reqwest`: HTTP client for API communication with JSON support and TLS support
- `serde`: Serialization/deserialization framework with derive macros
- `serde_json`: JSON parsing and manipulation support
- `anyhow`: Simplified error handling and context management
- `thiserror`: Custom error type derivation for structured error handling
- `dotenv`: Environment variable management from `.env` files
- `clap`: Command-line argument parsing with derive macros
- `colored`: Terminal color output for beautiful console display
- `chrono`: Date and time handling with serialization support
- `tracing`: Structured logging framework for debugging and monitoring
- `tracing-subscriber`: Logging subscriber for console output with environment filtering

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
# DeepSeek API Configuration
DEEPSEEK_API_KEY=your_deepseek_api_key_here
DEEPSEEK_BASE_URL=https://api.deepseek.com
```

**Advanced Configuration (Optional)**:
You can also set these environment variables for fine-tuning:
```env
DEEPSEEK_MODEL=deepseek-chat
DEEPSEEK_MAX_TOKENS=4096
DEEPSEEK_TEMPERATURE=0.7
DEEPSEEK_TIMEOUT=180
```

**Note**: The minimal `.env` file only requires `DEEPSEEK_API_KEY`. See `env.example` for the template.

**Note**: Command-line arguments override environment variable settings.

## Logging and debugging

The application uses structured logging with `tracing` for better debugging and monitoring:

### Log levels

Set the `RUST_LOG` environment variable to control logging output:

```bash
# Show info-level logs and above (default)
export RUST_LOG=info

# Show debug logs for detailed request/response information
export RUST_LOG=debug

# Show only warnings and errors
export RUST_LOG=warn

# Show all logs including trace level
export RUST_LOG=trace

# Target specific modules (example)
export RUST_LOG=deepseek_json::deepseek=debug,info
```

### Logging features

- 📊 **Retry visibility**: Warnings logged for retry attempts and backoff timing
- 🔍 **Structured logging**: Human-readable logs via `tracing`
- 🎯 **Configurable levels**: Control verbosity with `RUST_LOG`

## Project architecture

The application is built with a modular architecture for maintainability and extensibility:

### Core modules

- **`config.rs`**: Configuration management and environment variable handling
  - Validates configuration parameters
  - Provides sensible defaults
  - Supports both environment variables and command-line overrides

- **`deepseek.rs`**: DeepSeek API client and communication layer
  - Custom error types with `thiserror` integration
  - HTTP client with timeout and exponential backoff retry logic (3 attempts)
  - JSON response parsing and validation
  - Structured logging for request tracking and debugging
  - Advanced error mapping and network connectivity handling

- **`console.rs`**: User interface and terminal interaction
  - Colored output with emoji indicators
  - Interactive prompt handling with async I/O
  - Ctrl+C signal handling and request cancellation
  - Error display with contextual help and user-friendly messaging
  - Welcome and goodbye messages

- **`taskfinisher.rs`**: TaskFinisher-JSON flow and schema
  - System prompt builder with max-question limits and self-stop rule
  - Strongly-typed JSON structures for questions and the final artifact
  - Parser for assistant JSON into either clarifying questions or the final artifact

- **`lib.rs`**: Application orchestration and public API
  - Main `App` struct that coordinates all components
  - Initialization and configuration loading
  - Both interactive and single-query modes

- **`main.rs`**: CLI argument parsing and entry point
  - Command-line argument processing with `clap` 
  - Structured logging initialization with environment filtering
  - Single query mode handling
  - Application startup and error handling

### Design principles

- **Async-First**: Built on `tokio` for efficient I/O handling with signal management
- **Error Transparency**: Custom error types with retry logic and clear user feedback
- **Configuration Flexibility**: Multiple ways to configure via environment variables and CLI args
- **Observability**: Structured logging with `tracing` for debugging and monitoring
- **Separation of Concerns**: Each module has a single, well-defined responsibility
- **User Experience**: Prioritizes clear feedback, signal handling, and beautiful terminal output

## Error handling

The application features advanced error handling with custom error types and user-friendly messaging:

### Error types
- **ServerBusy**: Handles rate limiting and server overload scenarios
- **NetworkError**: DNS failures, connection issues, and network timeouts
- **Timeout**: Request timeouts with configurable duration
- **ApiError**: HTTP status code errors with context-aware messages
- **ParseError**: JSON parsing and response format issues
- **ConfigError**: Configuration validation and setup problems

### Error features
- 🎯 **Context-Aware Messages**: Different error types show appropriate user guidance
- 💡 **Recovery Suggestions**: Each error type includes helpful tips for resolution
- 🎨 **Color-Coded Display**: Errors are displayed with appropriate colors and emojis
- 🔍 **Detailed Logging**: Comprehensive error context for troubleshooting
- 🛡️ **Graceful Degradation**: Application continues running after recoverable errors

### Advanced retry logic
- 🔄 **Exponential Backoff**: Automatic retry with increasing delays (500ms, 1s, 2s)
- 🎯 **Smart Retry Conditions**: Only retries on server busy and network errors
- 📊 **Retry Logging**: Structured logs showing retry attempts and backoff timing
- ⚡ **Configurable Attempts**: Maximum of 3 attempts before giving up

### Cancellation and resilience
- Ctrl+C exits the app gracefully; during a request, it cancels the request and exits
- Automatic backoff and retries for transient failures (no pre-flight health checks)
