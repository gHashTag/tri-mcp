# RING.md — tri-mcp Ring Architecture

## Rings

| Ring | Name | Type | Port | Описание |
|------|------|------|------|----------|
| SR-00 | trios-mcp-sr00 | 🥈 SILVER | 3026 | HTTP + WebSocket сервер |
| SR-01 | trios-mcp-sr01 | 🥈 SILVER | — | Lighthouse bridge |
| SR-02 | trios-mcp-sr02 | 🥈 SILVER | stdio | MCP протокол |
| BR-XTASK | trios-mcp | 🥉 BRONZE | — | Launcher |

## Utility Crates

| Crate | Описание |
|-------|----------|
| browser-mcp-launcher | Запуск TypeScript сервера с env переменными |
| tunnel-launcher | Tailscale Funnel управление |
| health-check | 4 HTTP теста для проверки стека |

## PHI Loop

edit spec → seal hash → gen → test → verdict → experience → skill commit → git commit

## Port Configuration

- **HTTP Server:** 3026 (по умолчанию)
- **Tailscale Funnel:** 443 (HTTPS)
- **Discovery Range:** 3026-3035

## Quick Start

```bash
# Запуск TypeScript сервера
PORT=3026 AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 \
  npx tsx browser-tools-server/browser-connector.ts

# Запуск Funnel
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443 --bg 3026

# Health Check
cargo run -p health-check

# Perplexity MCP Config
cp config/mcp-perplexity.json ~/.perplexity/mcp.json
```
