# 🔥 Agent Quick Start - ONE SHOT Copy

## Server Status: ✅ RUNNING

```
MCP Server: RUNNING (discovered at 127.0.0.1:3025)
Browser Tools Server: RUNNING on port 3025 with AUTH
Tailscale Funnel: ⚠️ MANUAL SETUP REQUIRED
```

---

## 🚀 ONE COMMAND TO START EVERYTHING

```bash
cd /Users/playra/browser-tools-mcp && bun dev
```

---

## 🔐 Authentication Credentials

```
Username: perplexity
Password: test123
Base64: cGVycGxleGl0eTp0ZXN0MTIz
```

---

## 🔗 Important Links

| Purpose | URL |
|---------|-----|
| **MCP Server Source** | https://github.com/gHashTag/tri-mcp |
| **English Guide** | https://github.com/gHashTag/tri-mcp/blob/main/TASKS/POLICY-perplexity-auth-setup.md |
| **Russian Guide** | https://github.com/gHashTag/tri-mcp/blob/main/TASKS/perplexity-auth-test.md |
| **Tailscale Funnel** | `https://playras-macbook-pro-1.tail01804b.ts.net` (MANUAL SETUP) |

---

## ⚙️ Tailscale Funnel Setup (CRITICAL)

**Tailscale GUI Instructions:**

1. Click Tailscale icon in menu bar
2. Preferences → Funnel
3. Enable Funnel if not active
4. Set proxy to: `localhost:3025`

**Or via Terminal:**
```bash
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443 3025 --bg
```

---

## 🤖️ Chrome Extension Setup

1. Open Chrome DevTools (F12)
2. Go to **BrowserToolsMCP** tab
3. Server Connection Settings:
   - Host: `localhost`
   - Port: `3025`
   - **Enable Authentication:** ☑️
   - Username: `perplexity`
   - Password: `test123`
4. Click **Save Settings**

---

## 🤖️ Perplexity Agent MCP Config

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

---

## 🧪 Test Commands (for Perplexity)

| Command | Description |
|---------|-------------|
| "Take a screenshot" | Capture current browser tab |
| "Get console logs" | Get browser console output |
| "Run accessibility audit" | WCAG compliance audit |
| "Run performance audit" | Page performance analysis |

---

## ✅ Verification Checklist

| Component | Status |
|-----------|---------|
| Server (3025) | ✅ Running |
| Auth enabled | ✅ perplexity:test123 |
| MCP Server | ✅ Connected to 3025 |
| Tailscale Funnel | ⚠️ Manual setup needed |
| Chrome Extension | ⚠️ Manual setup needed |
| Perplexity Agent | ⏳ Configure with JSON above |

---

**Last Updated:** 2026-04-23
