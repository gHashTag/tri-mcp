# Perplexity Custom Connector — SR-02 Streamable HTTP Setup

This guide configures Perplexity (https://www.perplexity.ai/account/connectors) to talk to the
`browser-tools-server` over the new **`/mcp` Streamable HTTP** endpoint introduced by SR-02.

## 1. Endpoint

| Field | Value |
|---|---|
| URL | `https://<your-tailscale-funnel>/mcp` (e.g. `https://playras-macbook-pro-1.tail01804b.ts.net/mcp`) |
| Transport | **Streamable HTTP** |
| Methods | `POST /mcp` (JSON-RPC 2.0) — `initialize`, `tools/list`, `tools/call` |
| `GET /mcp` | Returns 405 (stateless mode — no SSE channel) |

## 2. Authentication

The server's auth middleware (after PR-2, see `[SR-02]` follow-up) accepts
**three** equivalent header shapes — use whichever your MCP client supports:

| Perplexity UI choice | Header sent by client | Server behaviour |
|---|---|---|
| **Basic Auth** | `Authorization: Basic <base64(user:pass)>` | ✅ accepted (RFC 7617) |
| **API Key** field set to `perplexity:test123` | `Authorization: Bearer perplexity:test123` | ✅ accepted (literal `user:pass` after `Bearer`) |
| API Key field set to a pre-encoded base64 token | `Authorization: Bearer <base64(user:pass)>` | ✅ accepted (decoded, then matched) |

| Field | Value |
|---|---|
| Username | `$AUTH_USERNAME` (default `admin`, recommended `perplexity`) |
| Password | `$AUTH_PASSWORD` (set in your local env) |

In the current Perplexity Custom Connector UI, the Authentication dropdown
offers **None** and **API Key** — there is no "Basic Auth" option. Choose
**API Key** and enter the literal string `perplexity:test123` (or whatever
`AUTH_USERNAME:AUTH_PASSWORD` you set). The server now decodes both forms.

The same auth header guards every browser-tools-server route (except
`/.identity` and `/.port`), so this single credential covers `/mcp` plus
the 14 internal REST routes the tools bridge to.

## 3. Tools Exposed (14)

Bridged 1:1 from the existing stdio MCP server (`browser-tools-mcp/mcp-server.ts`):

| Tool | Internal route |
|---|---|
| `getConsoleLogs` | `GET /console-logs` |
| `getConsoleErrors` | `GET /console-errors` |
| `getNetworkErrors` | `GET /network-errors` |
| `getNetworkLogs` | `GET /network-success` |
| `takeScreenshot` | `POST /capture-screenshot` |
| `getSelectedElement` | `GET /selected-element` |
| `wipeLogs` | `POST /wipelogs` |
| `runAccessibilityAudit` | `POST /accessibility-audit` |
| `runPerformanceAudit` | `POST /performance-audit` |
| `runSEOAudit` | `POST /seo-audit` |
| `runBestPracticesAudit` | `POST /best-practices-audit` |
| `runNextJSAudit` | composite (guidance text) |
| `runDebuggerMode` | composite (guidance text) |
| `runAuditMode` | composite (guidance text) |

## 4. Quick smoke test

```bash
BASE="https://<funnel-host>/mcp"
AUTH=$(printf '%s' 'perplexity:test123' | base64)

# initialize
curl -sS -X POST "$BASE" \
  -H "Authorization: Basic $AUTH" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"0.0.1"}}}'

# tools/list — expect 14 entries
curl -sS -X POST "$BASE" \
  -H "Authorization: Basic $AUTH" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# tools/call wipeLogs — should return {status:"ok"}
curl -sS -X POST "$BASE" \
  -H "Authorization: Basic $AUTH" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wipeLogs","arguments":{}}}'
```

Responses use `text/event-stream` (single `event: message` frame per
JSON-RPC reply) — Perplexity's Streamable HTTP transport handles this
natively.

## 5. Common errors

| Symptom | Cause | Fix |
|---|---|---|
| `FETCHER_HTML_STATUS_CODE_ERROR` | Pre-PR-2 server: Perplexity sent `Authorization: Bearer …` while server only accepted Basic | Pull `main` past PR-2 — Bearer literal `user:pass` and `Bearer <base64>` are now both accepted |
| 401 on every call | No Authorization header | Set the API Key field in Perplexity UI to `user:pass` |
| 403 on every call | Wrong username/password | Match exactly to `AUTH_USERNAME` / `AUTH_PASSWORD` env |
| 400 `Malformed Authorization header` | Header missing the token after the scheme | Re-enter the API Key in the connector UI |
| 400 `Unsupported auth scheme: …` | Some other scheme (e.g. `Token`, `Digest`) | Use Basic or API Key (Bearer) only |
| `tools/list` returns 6 tools | Pointing at the Railway public MCP, not your local funnel | Update URL to your Tailscale Funnel host + `/mcp` |
| `Method Not Allowed (stateless /mcp)` | Client tried `GET /mcp` for SSE stream | Only `POST /mcp` is supported in stateless mode |

## 6. Local server start

```bash
cd browser-tools-server
PORT=3026 AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npm start
# → "[MCP] /mcp will expose 14 tools: …"
```

Then `tailscale funnel 3026` (or your existing funnel) makes the endpoint
reachable from Perplexity. Funnel terminates TLS — Basic Auth still rides
inside.
