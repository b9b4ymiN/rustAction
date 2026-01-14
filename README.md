# KS Forward Video Processor

> Automated YouTube transcript summarization with AI-powered analysis and Discord integration

[![Rust](https://img.shields.io/badge/rust-1.83+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 Features

- **Automated Video Processing**: Fetches and processes the latest KS Forward videos
- **AI-Powered Summarization**: Leverages AI to generate concise Investor's Notes
- **Smart Discord Integration**: Automatically posts formatted summaries to Discord
- **Transcript Caching**: Reduces API calls with intelligent caching
- **Retry Logic**: Handles transient failures with automatic retries
- **Production-Ready**: Structured logging, error handling, and Docker support
- **GitHub Actions**: Automated daily runs with monitoring

## 📋 Prerequisites

- Rust 1.83+
- Docker (optional, for containerized deployment)
- Required API keys:
  - YouTube Data API v3
  - Supabase (or your transcript API)
  - AI Service API
  - Discord Webhook URL

## 🔧 Configuration

Create a `.env` file in the project root:

```bash
# API Configuration
API_URL=https://api.example.com
TOKEN=your_api_token_here

# YouTube Configuration
YOUTUBE_API_KEY=your_youtube_api_key
KSFORWORD_CHANNEL_ID=UCxxxxxxxxxxxxxxxxxx

# Transcript API
SUPABASE_API_KEY=your_supabase_api_key

# AI Service
MY_AI_API_URL=http://localhost:8000/chat

# Discord Integration
DISCORD_KS_BOT_TOKEN=https://discord.com/api/webhooks/...

# Feature Flags
USE_MOCK_DATA=false
RUST_LOG=info
```

> **Security**: Keep secrets out of the repo. In GitHub Actions, set them under **Settings → Secrets and variables → Actions** with the same names.

## 🏃 Local Development

### Using Cargo

```bash
# Install dependencies
cargo build

# Run the processor
cargo run --release

# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Check code
cargo clippy
cargo fmt
```

### Using Docker

```bash
# Build the container
docker build -t schrust:latest .

# Run with docker-compose
docker-compose up -d

# View logs
docker-compose logs -f

# Run manually
docker run --rm \
  -e API_URL="$API_URL" \
  -e YOUTUBE_API_KEY="$YOUTUBE_API_KEY" \
  -e DISCORD_KS_BOT_TOKEN="$DISCORD_KS_BOT_TOKEN" \
  schrust:latest
```

## 📦 Deployment

### GitHub Actions

The workflow automatically runs daily at 4:00 AM Bangkok time (UTC+7).

**Workflow File**: `.github/workflows/daily-run.yml`

**Triggers**:
- Scheduled: `cron: "0 21 * * *"` (9 PM UTC = 4 AM Bangkok)
- Manual: `workflow_dispatch`

**Required Secrets**:
- `API_URL`
- `TOKEN`
- `YOUTUBE_API_KEY`
- `SUPABASE_API_KEY`
- `KSFORWORD_CHANNEL_ID`
- `MY_AI_API_URL`
- `DISCORD_KS_BOT_TOKEN`
- `USE_MOCK_DATA` (optional, defaults to `false`)

To manually trigger:
1. Go to **Actions** tab in GitHub
2. Select **"KS Forward Daily Runner"**
3. Click **"Run workflow"**

### Docker Deployment

```bash
# Pull latest image
docker pull your-registry/schrust:latest

# Run with environment variables
docker run -d \
  --name ks-forward-processor \
  --env-file .env \
  -v $(pwd)/transcript_cache:/app/transcript_cache \
  your-registry/schrust:latest
```

## 🏗️ Project Structure

```
schRust/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types and handling
│   ├── models/              # Data models
│   │   ├── youtube_transcript.rs
│   │   ├── myAI_response.rs
│   │   └── discord.rs
│   └── services/            # Business logic
│       ├── http_client.rs   # Optimized HTTP client
│       ├── youtube_service.rs
│       ├── supabase_service.rs
│       ├── myAI_service.rs
│       ├── discord_service.rs
│       └── ksForword_service.rs
├── .github/workflows/       # CI/CD workflows
├── transcript_cache/        # Cached transcripts
├── Cargo.toml
├── Dockerfile
└── docker-compose.yml
```

## 🔍 Monitoring

### Logs

The application uses structured logging with `tracing`:

```bash
# View logs in Docker
docker-compose logs -f schrust

# Filter by level
RUST_LOG=warn cargo run

# Filter by module
RUST_LOG=schRust::services::myAI_service=debug cargo run
```

### Health Checks

```bash
# Check if container is healthy
docker ps --filter name=ks-forward-processor

# Detailed health status
docker inspect ks-forward-processor --format='{{.State.Health.Status}}'
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_error_categories
```

## ⚡ Performance

- **Connection Pooling**: Reuses HTTP connections
- **Caching**: Transcripts cached locally to reduce API calls
- **Optimized Binary**: Release builds with LTO and stripping
- **Async Runtime**: Tokio-based async I/O

### Build Optimizations

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization at cost of compile time
strip = true         # Remove debug symbols
panic = "abort"      # Reduce binary size
```

## 🐛 Troubleshooting

### Build Errors

```bash
# Update Rust toolchain
rustup update stable

# Clean build artifacts
cargo clean

# Rebuild
cargo build --release
```

### Runtime Errors

1. **Missing environment variables**: Ensure all required variables are set in `.env`
2. **API failures**: Check API keys and rate limits
3. **Discord webhook fails**: Verify webhook URL and message format

### Debug Mode

```bash
# Enable detailed logging
RUST_LOG=debug RUST_BACKTRACE=1 cargo run

# Enable trace for specific module
RUST_LOG=schRust::services::myAI_service=trace cargo run
```

## 🔒 Security

- All secrets stored as GitHub Secrets
- Configuration validation at startup
- Sensitive data masked in logs
- Non-root Docker user
- Minimal attack surface

## 📝 License

This project is licensed under the MIT License.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📧 Support

For issues and questions:
- Open an issue on GitHub
- Check existing documentation
- Review logs for error details

---

Built with ❤️ using [Rust](https://www.rust-lang.org/)
