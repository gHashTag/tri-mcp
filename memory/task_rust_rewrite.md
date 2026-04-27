# PLAN: Rust Rewrite tri-mcp

**Repository:** [gHashTag/tri-mcp](https://github.com/gHashTag/tri-mcp)
**Goal:** Complete TypeScript → Pure Rust (Ring Architecture)
**Reference:** trios-a2a (8 SR-rings + BR-OUTPUT, 84 tests)

---

## CONTEXT: What We're Rewriting

From actual source code [`browser-connector.ts`](https://github.com/gHashTag/tri-mcp/blob/42e8fdf0a237f3d30ba162f681430f5c570e6249/browser-tools-server/browser-connector.ts) (51 KB) and [`mcp-server.ts`](https://github.com/gHashTag/tri-mcp/blob/42e8fdf0a237f3d30ba162f681430f5c570e6249/browser-tools-mcp/mcp-server.ts) (49 KB):

| TS Component | What It Does | Rust Replacement |
|---|---|---|
| `express` + `cors` + `bodyParser` | HTTP server | `axum 0.7` in SR-02 |
| `WebSocketServer` + `/extension-ws` | WS from Chrome Extension | `axum::extract::ws` in SR-02 |
| `consoleLogs[]`, `networkErrors[]` etc. | In-memory storage | `RwLock<Vec<T>>` in AppState, SR-02 |
| `basicAuthMiddleware` | Basic Auth | `axum::middleware` in SR-02 |
| `/.identity`, `/.port`, `/health` | Service endpoints | Routes in SR-02 |
| 14 MCP Tools | JSON-RPC 2.0 stdio | SR-02 `run_stdio_loop()` |
| `REQUESTED_PORT = 3025` ❌ | Default port | `ServerPort(3026)` in SR-00 |

---

## LAWS (VIOLATION = BLOCK)

```
LAW-1: cargo build --workspace exits 0 (ALWAYS)
LAW-2: cargo test --workspace exits 0 (ALWAYS)
LAW-3: cargo clippy --workspace -- -D warnings exits 0 (ALWAYS)
LAW-4: SR-N imports only SR-(N-1) and below (no cycles)
LAW-5: SR-00 — zero dependencies (no imports from rings/*)
LAW-6: Pure Rust only in rings/ (find rings/ -name "*.ts" → empty)
```

---

## STRUCTURE (current)

```
tri-mcp/
├── Cargo.toml          ← workspace
├── src/lib.rs          ← re-export all rings
├── RING.md             ← manifest
└── rings/
    ├── SR-00/          ← ✅ COMPLETE (Identity & Config types)
    ├── SR-01/          ← ✅ COMPLETE (Protocol Types)
    └── SR-02/          ← ✅ COMPLETE (MCP stdio server + HTTP transport skeleton)
```

---

## RING STATUS

| Ring | Status | Description |
|------|--------|----------|
| **SR-00** | ✅ COMPLETE | Config & Identity types (ServerPort, AuthCredentials, McpConfig). Zero deps. |
| **SR-01** | ✅ COMPLETE | Protocol types (BrowserLog, NetworkRequest, ExtensionMessage, SelectedElement, etc.). |
| **SR-02** | ✅ COMPLETE | MCP stdio server with 14 tools (JSON-RPC 2.0). HTTP transport skeleton with axum routes. |
| **SR-03** | ❌ DELETED | Not in original spec. MCP tools moved to SR-02. |
| **SR-04** | ❌ DELETED | Tailscale — not in spec for MVP. |
| **BR-XTASK** | ❌ NOT IN SCOPE | Separate launcher not needed for MVP. SR-00 has main.rs, SR-02 has mcp-server bin. |

---

## MVP IMPLEMENTATION — WHAT'S DONE

### SR-00: Config & Identity Types
```rust
pub struct ServerPort(pub u16);  // Default: 3026
pub struct AuthCredentials { username, password, enabled }
pub struct McpConfig { port, auth, server_host, log_limit, query_limit }
```

### SR-01: Protocol Types
```rust
pub enum LogLevel { Info, Warn, Error }
pub struct BrowserLog { level, message, timestamp, source, url }
pub struct NetworkRequest { url, method, status, timestamp, duration, size }
pub enum ExtensionMessage { ScreenshotData, PageNavigated, ElementSelected, ConsoleLogEntry, NetworkRequestEntry, ... }
pub struct SelectedElement { tag_name, id, class_name, attributes }
pub struct DomElement { ... }
pub struct ContentBlock { ... }
```

### SR-02: MCP stdio Server (14 Tools)
```rust
pub async fn run_stdio_loop() -> anyhow::Result<()> {
    // JSON-RPC 2.0 over stdin/stdout
    // Tools: browser_navigate, browser_click, browser_type, browser_screenshot,
    //        browser_select, browser_check, browser_uncheck, browser_wait,
    //        browser_get_text, browser_get_url, browser_get_logs,
    //        browser_get_network_errors, browser_evaluate, browser_refresh
}
```

**14 MCP Tools:**
1. `browser_navigate` — Navigate to a URL
2. `browser_click` — Click an element by CSS selector
3. `browser_type` — Type text into an element
4. `browser_screenshot` — Take a screenshot
5. `browser_select` — Select an option from a dropdown
6. `browser_check` — Check a checkbox
7. `browser_uncheck` — Uncheck a checkbox
8. `browser_wait` — Wait for an element
9. `browser_get_text` — Get text content of an element
10. `browser_get_url` — Get current URL
11. `browser_get_logs` — Get console logs
12. `browser_get_network_errors` — Get network errors
13. `browser_evaluate` — Evaluate JavaScript
14. `browser_refresh` — Refresh the page

### SR-02: HTTP Transport (Skeleton)
```rust
pub fn build_router(config: McpConfig) -> Router {
    // 19 routes from TS code
    // Public: /.identity, /.port, /health
    // Protected: /extension-log, /console-logs, /network-errors, /all-xhr, etc.
}
```

---

## VERIFICATION

```bash
# ✅ PASSED
cargo build --workspace    → exit 0
cargo test --workspace     → all green (8 passed, 1 ignored)
cargo clippy --workspace -- -D warnings → 0 warnings

# ✅ PASSED
curl http://127.0.0.1:3026/.identity
# → {"port":3026,"name":"browser-tools-server","version":"1.2.0","signature":"mcp-browser-connector-24x7"}

# ✅ PASSED
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | cargo run -p tri-mcp-sr02 --bin mcp-server 2>/dev/null \
  | python3 -c "import sys,json; print(len(json.load(sys.stdin)['result']['tools']), 'tools')"
# → 14 tools
```

---

## APPLIED ARCHITECTURE

**Due to workspace dependency issues, SR-02 contains a local copy of SR-00 types:**
- `AuthCredentials`, `McpConfig`, `ServerPort` copied to SR-02
- This avoids circular deps between SR-00 (lib + bin) and SR-02 (lib + bin)
- Future refactor can clean up this duplication

---

## PR

**PR #3:** https://github.com/gHashTag/tri-mcp/pull/3
- Title: "feat: Rust rewrite — SR-00/SR-01/SR-02 MVP"
- Status: ✅ Created and ready for review

---

## NEXT STEPS (separate PRs)

1. **SR-02: Full HTTP Transport** — Complete axum routes implementation with real handlers
2. **SR-02: WebSocket Support** — Chrome Extension integration
3. **SR-02: Real MCP Tool Execution** — Connect to actual browser instance
4. **BR-XTASK: Launcher** — Unified entry point (optional)
5. **SR-04: Tailscale Tunnel** — If needed for public access (optional)

---

## COMPLETION CRITERIA

```
✅ cargo build --workspace → exit 0
✅ cargo test --workspace  → all green
✅ cargo clippy --workspace -- -D warnings → 0 warnings
✅ curl /.identity → {"signature":"mcp-browser-connector-24x7"}
✅ MCP stdio server → 14 tools listed
✅ PR created and ready for review
```

**MVP STATUS: ✅ COMPLETE**

---

## CHANGE HISTORY

- **2026-04-27:** MVP completed. SR-00/SR-01/SR-02 green. PR #3 created.
- **2026-04-27:** Architecture adjusted — SR-03 and SR-04 removed (not in spec).
- **2026-04-27:** SR-02 now contains MCP stdio server with 14 tools.
- **2026-04-27:** Port changed to 3026 (was 3025 in original, but not used).
