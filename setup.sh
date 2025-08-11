#!/bin/bash

# DeepSeek JSON Chat Application Setup Script

echo "🤖 DeepSeek JSON Chat Application Setup"
echo "======================================="

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "📝 Creating .env file from template..."
    cp env.example .env
    echo "✅ .env file created"
    echo "⚠️  Please edit .env and add your DeepSeek API key!"
    echo ""
else
    echo "✅ .env file already exists"
fi

# Check if API key is set
if grep -q "your_deepseek_api_key_here" .env 2>/dev/null; then
    echo "⚠️  Warning: Please update your DEEPSEEK_API_KEY in .env file"
    echo "   Visit https://platform.deepseek.com to get your API key"
    echo ""
fi

# Build the project
echo "🔨 Building the project..."
if cargo build --release; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Ready to run! Use one of the following commands:"
    echo "   cargo run                    # Run in development mode"
    echo "   ./target/release/deepseek_json  # Run optimized binary"
    echo ""
    echo "📚 Usage:"
    echo "   1. Make sure your API key is set in .env"
    echo "   2. Run the application"
    echo "   3. Enter your questions"
    echo "   4. Type 'quit' or 'exit' to stop"
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi

