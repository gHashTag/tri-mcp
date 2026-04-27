# Perplexity Agent Authentication Setup Guide

## Objective

Setup and test authentication for browser-tools-mcp server accessible to Perplexity agent via Tailscale Funnel tunnel.

## Architecture

```
Perplexity Agent (WS)
  → wss:// + Basic Auth headers
    → Tailscale Funnel (HTTPS :443)
      → browser-tools-server (localhost:3026)
        → Chrome Extension (WebSocket)
```

## Prerequisites

1. **Tailscale** installed from App Store (NOT Homebrew!)
2. **Chrome** with BrowserToolsMCP extension installed
3. **Node.js** and npm available
4. **Perplexity agent** with MCP support enabled

---

## Step-by-Step Instructions

### Step 1: Configure Authentication Credentials

Copy `.env.example` to `.env` and fill in:

```
AUTH_USERNAME=perplexity
AUTH_PASSWORD=test123
PORT=3026
TAILSCALE_URL=https://playras-macbook-pro-1.tail01804b.ts.net
```

---

### Step 2: Start browser-tools-server (local fork)

```bash
cd /Users/playra/tri-mcp
PORT=3026 AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx tsx browser-tools-server/browser-connector.ts
```

**Expected output:**
```
Starting Browser Tools Server...
Requested port: 3026
Found available port: 3026

=== Browser Tools Server Started ===
Aggregator listening on http://0.0.0.0:3026
```

---

### Step 3: Configure Chrome Extension

1. Open Chrome DevTools (F12)
2. Navigate to **BrowserToolsMCP** tab
3. In **Server Connection Settings**:
   - Server Host: `localhost`
   - Server Port: `3026`
   - **Enable Authentication:** ☑️ (check the box)
   - **Username:** `perplexity`
   - **Password:** `test123`
4. Click **Save Settings**
5. Verify status shows `Connected` ✅

---

### Step 4: Start Tailscale Funnel

**CRITICAL:** Use App Store version of Tailscale, NOT Homebrew!

```bash
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443 --bg 3026
```

**Note:** Argument order matters — `--bg` before port number.

**Expected output:**
```
Funnel is on.
Your Funnel is available at:
* https://playras-macbook-pro-1.tail01804b.ts.net
```

---

### Step 5: Verify with health-check crate

```bash
cd /Users/playra/tri-mcp
cargo run -p health-check
```

**Expected output:**
```
✅ Public /.identity → 200
✅ No auth /console-logs → 401
✅ Valid auth /console-logs → 200
✅ Wrong auth /console-logs → 403

Results: 4 passed, 0 failed
```

---

### Step 6: Configure Perplexity Agent

File: `~/.perplexity/mcp.json`

```json
{
  "mcpServers": [{
    "name": "browser-tools",
    "url": "wss://playras-macbook-pro-1.tail01804b.ts.net",
    "transport": "websocket",
    "headers": {
      "Authorization": "Basic cGVycGxleGl0eTp0ZXN0MTIz"
    }
  }]
}
```

To regenerate Base64 when password changes:
```bash
echo -n "perplexity:test123" | base64
# Output: cGVycGxleGl0eTp0ZXN0MTIz
```

---

### Step 7: Test with Perplexity

1. **"Take a screenshot"** — captures current browser tab
2. **"Get console logs"** — returns browser console output
3. **"Run accessibility audit"** — runs WCAG audit

---

## Troubleshooting

### Issue: Tailscale funnel invalid argument

**Wrong:**
```bash
tailscale funnel --https=443 3026 --bg   # ❌ --bg must come before port
```
**Correct:**
```bash
tailscale funnel --https=443 --bg 3026   # ✅
```

### Issue: Server starts on wrong port

The npm version ignores `PORT` env. Use local fork with `tsx`:
```bash
PORT=3026 AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx tsx browser-tools-server/browser-connector.ts
```

### Issue: Auth not enforced

The npm-published `@agentdeskai/browser-tools-server` may not include Basic Auth. Always run from **local tri-mcp fork** which has auth in `browser-tools-server/browser-connector.ts`.

---

## Current Configuration

| Setting | Value |
|----------|--------|
| Tunnel URL | `https://playras-macbook-pro-1.tail01804b.ts.net` |
| Server Port | `3026` |
| Auth Username | `perplexity` |
| Auth Password | `test123` |
| Base64 Auth | `cGVycGxleGl0eTp0ZXN0MTIz` |
| Protocol | WebSocket (`wss://`) |
| Server source | Local fork `tri-mcp/browser-tools-server/` |

---

## Status

| Component | Status |
|-----------|---------|
| Server | ✅ Running on port 3026 with Basic Auth |
| Tailscale Funnel | ✅ `wss://playras-macbook-pro-1.tail01804b.ts.net` → 3026 |
| Authentication | ✅ 4/4 health checks passed (401/200/403) |
| Chrome Extension | ⚠️ Manual configuration required (port 3026) |
| Perplexity Connection | ✅ Configured via `~/.perplexity/mcp.json` |

---

**Last Updated:** 2026-04-27
**Version:** 1.3.0
