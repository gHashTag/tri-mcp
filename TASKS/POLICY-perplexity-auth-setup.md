# Perplexity Agent Authentication Setup Guide

## Objective

Setup and test authentication for browser-tools-mcp server accessible to Perplexity agent via Tailscale Funnel tunnel.

## Architecture

```
Perplexity Agent (WS)
  → wss:// + Basic Auth headers
    → Tailscale Funnel (HTTPS :443)
      → browser-tools-server (localhost:3025)
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

Set environment variables for the server:

```bash
export AUTH_USERNAME=perplexity
export AUTH_PASSWORD=test123
```

Or pass them directly:

```bash
AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx @agentdeskai/browser-tools-server@latest
```

**Credentials:**
- Username: `perplexity`
- Password: `test123`
- Base64 (for HTTP headers): `cGVycGxleGl0eTp0ZXN0MTIz`

---

### Step 2: Start browser-tools-server

Start the server with authentication enabled:

```bash
AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx @agentdeskai/browser-tools-server@latest
```

**Expected output:**
```
Starting Browser Tools Server...
Requested port: 3025
Found available port: 3025

=== Browser Tools Server Started ===
Aggregator listening on http://0.0.0.0:3025

For local access use: http://localhost:3025
```

**Note:** Server will use `AUTH_USERNAME=perplexity` and `AUTH_PASSWORD=test123` by default.

---

### Step 3: Configure Chrome Extension

1. Open Chrome DevTools (F12)
2. Navigate to **BrowserToolsMCP** tab
3. In **Server Connection Settings**:
   - Server Host: `localhost`
   - Server Port: `3025`
   - **Enable Authentication:** ☑️ (check the box)
   - **Username:** `perplexity`
   - **Password:** `test123`
4. Click **Save Settings**
5. Verify status shows `Connected` ✅

---

### Step 4: Start Tailscale Funnel

**CRITICAL:** Use App Store version of Tailscale, NOT Homebrew! The App Store version has better WebSocket support.

Start the tunnel:

```bash
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443 3025 --bg
```

Or via GUI:
1. Open Tailscale.app from menu bar
2. Go to **Preferences** → **Funnel**
3. Click **Turn on Funnel**
4. Set proxy to: `http://127.0.0.1:3025`

**Expected output:**
```
Funnel is on.
Your Funnel is available at:
* https://playras-macbook-pro-1.tail01804b.ts.net
```

**Save the tunnel URL** - you'll need it for Perplexity configuration.

---

### Step 5: Verify Tunnel

Test that the tunnel is working:

```bash
# Test identity endpoint (should always work - no auth required)
curl https://playras-macbook-pro-1.tail01804b.ts.net/.identity

# Expected response:
{"port":3025,"name":"browser-tools-server","version":"1.2.0",...}

# Test with correct credentials
curl -u perplexity:test123 https://playras-macbook-pro-1.tail01804b.ts.net/console-logs

# Test with wrong credentials (should return 403)
curl -u wrong:creds https://playras-macbook-pro-1.tail01804b.ts.net/console-logs
```

**Expected results:**

| Test | Credentials | HTTP Code | Status |
|-------|-------------|-------------|---------|
| /.identity | none | 200 | ✅ Public endpoint |
| /console-logs | none | 401 | ✅ Auth required |
| /console-logs | perplexity:test123 | 200 | ✅ Auth success |
| /console-logs | wrong:creds | 403 | ✅ Invalid credentials |

---

### Step 6: Configure Perplexity Agent

Perplexity agent connects via WebSocket with Basic Auth headers.

**MCP Configuration:**

```json
{
  "mcpServers": [{
    "url": "wss://playras-macbook-pro-1.tail01804b.ts.net",
    "transport": "websocket",
    "headers": {
      "Authorization": "Basic cGVycGxleGl0eTp0ZXN0MTIz"
    }
  }]
}
```

**How to generate Base64 credentials:**

```bash
echo -n "perplexity:test123" | base64
# Output: cGVycGxleGl0eTp0ZXN0MTIz
```

---

### Step 7: Test with Perplexity

Once connected, test these commands:

1. **"Take a screenshot"** - Should capture current browser tab
2. **"Get console logs"** - Should return browser console output
3. **"Run accessibility audit"** - Should run WCAG audit on current page

---

## Troubleshooting

### Issue: "Port 3025 is in use"

**Solution:** Find and kill existing process:

```bash
# Find what's using the port
lsof -i :3025

# Kill the process
kill <PID>

# Or kill all browser-tools-server instances
pkill -f "browser-tools-server"
```

---

### Issue: Tailscale Funnel shows wrong port

**Solution:** The Funnel may be using old configuration. Reset it:

```bash
# Stop existing funnel
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel stop

# Reset configuration
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel reset

# Start with correct port
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443 3025 --bg
```

Or use Tailscale GUI:
1. Open Tailscale.app → Preferences → Funnel
2. Click "Reset Funnel"
3. Set proxy to: `http://127.0.0.1:3025`

---

### Issue: Authentication not working (401 on all requests)

**Solution:** Verify server is running with correct environment variables:

```bash
# Check running server
ps aux | grep "browser-tools-server"

# Kill and restart with auth variables
pkill -f "browser-tools-server"
AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx @agentdeskai/browser-tools-server@latest
```

---

### Issue: Perplexity cannot connect via WebSocket

**Solution:** Tailscale App Store version required. Homebrew version has poor WebSocket support.

```bash
# Verify you're using App Store version
/Applications/Tailscale.app/Contents/MacOS/Tailscale --version

# NOT this (Homebrew version)
/opt/homebrew/bin/tailscale --version
```

---

## Key Files

| File | Purpose |
|-------|-----------|
| `browser-tools-server/browser-connector.ts` | Authentication logic (lines 247-281) |
| `chrome-extension/panel.js` | Extension auth settings UI |
| `chrome-extension/devtools.js` | WebSocket connection management |

---

## Quick Reference Commands

```bash
# Start server with auth
AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx @agentdeskai/browser-tools-server@latest

# Start tunnel (App Store version)
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443 3025 --bg

# Check tunnel status
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel status

# Reset tunnel
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel reset

# Test authentication
curl -u perplexity:test123 http://localhost:3025/.identity

# Test tunnel
curl https://<YOUR_TAILSCALE_URL>/.identity
```

---

## Current Configuration

| Setting | Value |
|----------|--------|
| Tunnel URL | `https://playras-macbook-pro-1.tail01804b.ts.net` |
| Server Port | `3025` |
| Auth Username | `perplexity` |
| Auth Password | `test123` |
| Base64 Auth | `cGVycGxleGl0eTp0ZXN0MTIz` |
| Protocol | WebSocket (wss://) |

---

## Status

| Component | Status |
|-----------|---------|
| Server | ✅ Running on port 3025 with auth |
| Tailscale Funnel | ⚠️ Manual configuration required |
| Authentication | ✅ Tested via curl (401/200/403 responses) |
| Chrome Extension | ⚠️ Manual configuration required |
| Perplexity Connection | ⏳ Pending configuration |

---

**Last Updated:** 2026-04-23
**Version:** 1.2.0
