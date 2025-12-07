# KS Forward Daily Summary (Rust)

Rust async service that fetches the latest **KS Forward** YouTube video, pulls its transcript, asks your AI service to summarize it, and posts the result to Discord. A GitHub Actions workflow (`.github/workflows/run.yml`) schedules it to run Mon–Fri at 11:00 Asia/Bangkok.

## What it does
- Searches the KS Forward channel (`KSFORWORD_CHANNEL_ID`) for the newest video titled "KS Forward"
- Gets the transcript via the SupaData transcript API (or local mock data when enabled)
- Sends the full transcript to your AI endpoint (`MY_AI_API_URL`) for summarization
- Posts the AI answer to a Discord webhook, chunking long messages into multiple embeds

## Prerequisites
- Rust toolchain (stable) with Cargo
- Network access to YouTube Data API, SupaData transcript API, your AI endpoint, and Discord webhook

## Configuration (.env)
Create a `.env` file (kept out of git) or set these as GitHub Secrets for CI:

| Variable | Description |
| --- | --- |
| `API_URL` | Reserved/custom API base (loaded but currently unused in code) |
| `TOKEN` | Reserved/custom token (loaded but currently unused in code) |
| `YOUTUBE_API_KEY` | YouTube Data API key for channel search |
| `SUPABASE_API_KEY` | SupaData transcript API key (`x-api-key`) |
| `KSFORWORD_CHANNEL_ID` | YouTube channel ID to monitor |
| `USE_MOCK_DATA` | `true` to use `src/mock_data/example_transcript.json` instead of calling transcript API |
| `MY_AI_API_URL` | Your AI summarizer endpoint (expects JSON with `persona`, `user_id`, `messages`) |
| `DISCORD_KS_BOT_TOKEN` | Discord webhook URL for posting the summary |

> Keep secrets out of the repo. In GitHub Actions, set them under **Settings ? Secrets and variables ? Actions** with the same names.

## Run locally
```bash
cargo run --release
```
- With mock transcript (offline-friendly): set `USE_MOCK_DATA=true` in `.env`.

## GitHub Actions (scheduled run)
- Workflow: `.github/workflows/run.yml`
- Triggers: `cron: "0 4 * * 1-5"` with `TZ: Asia/Bangkok` ? runs 11:00 Mon–Fri; also supports manual `workflow_dispatch`
- Steps: checkout ? create `.env` from Secrets ? install Rust stable ? `cargo build --release` ? `cargo run --release`

## Project structure
- `src/main.rs` — entrypoint; loads config and kicks off KS Forward job
- `src/config.rs` — loads environment variables
- `src/services/` — YouTube search, transcript fetch, AI call, Discord webhook, mock helpers
- `src/models/` — request/response models for APIs and Discord payloads
- `src/mock_data/example_transcript.json` — sample transcript used when `USE_MOCK_DATA=true`

## Notes
- Discord embed description limit is handled by chunking to avoid failures.
- If keys were ever committed, rotate them and replace with Secrets before deploying.