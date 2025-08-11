# DeepSeek JSON Chat Application

A Rust application that sends requests to DeepSeek's API and receives structured JSON responses. The application automatically adds JSON format specifications to user prompts and parses the structured responses for console display.

## Features

- 🤖 **DeepSeek Integration**: Direct integration with DeepSeek's chat API
- 📋 **Structured Responses**: Enforces JSON format with predefined fields
- 🎯 **Field Extraction**: Automatically parses and displays JSON fields
- 🛡️ **Error Handling**: Comprehensive error handling for network and parsing issues
- 🔄 **Interactive Loop**: Continuous chat interface until user exits

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

1. **Clone and navigate to the project**:
   ```bash
   git clone <repository-url>
   cd deepseek_json
   ```

2. **Set up environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env and add your DeepSeek API key
   ```

3. **Get your DeepSeek API key**:
   - Visit [DeepSeek Platform](https://platform.deepseek.com)
   - Create an account and generate an API key
   - Add the key to your `.env` file

4. **Build and run**:
   ```bash
   cargo run
   ```

## Usage

1. Run the application: `cargo run`
2. Enter your questions when prompted
3. The application will:
   - Add JSON format instructions to your prompt
   - Send the request to DeepSeek
   - Parse the JSON response
   - Display structured fields in the console
4. Type `quit` or `exit` to stop

## Example

```
💬 Enter your question: Tell me about Rust programming language

🔄 Sending request to DeepSeek...

📋 Structured Response:
┌─────────────────────────────────────────────────────────────
│ 🏷️  Title: Rust Programming Language Overview
│ 📝 Description: A systems programming language focused on safety and performance
│ 📄 Content: Rust is a modern systems programming language...
│ 🏪 Category: Programming Languages
│ ⏰ Timestamp: 2024-01-15T10:30:00Z
│ 🎯 Confidence: 0.95
└─────────────────────────────────────────────────────────────
```

## Dependencies

- `tokio`: Async runtime for handling HTTP requests
- `reqwest`: HTTP client for API communication
- `serde`: Serialization/deserialization framework
- `serde_json`: JSON parsing support
- `anyhow`: Error handling utilities
- `dotenv`: Environment variable management

## Configuration

Environment variables:
- `DEEPSEEK_API_KEY`: Your DeepSeek API key (required)
- `DEEPSEEK_BASE_URL`: API base URL (optional, defaults to https://api.deepseek.com)

## Error Handling

The application handles various error scenarios:
- Missing or invalid API key
- Network connectivity issues
- API rate limiting or errors
- Invalid JSON responses
- Parsing failures

All errors are displayed with helpful messages to guide troubleshooting.

