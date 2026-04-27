# AGENT CONTEXT — Rust Rewrite Mission

## Твоя задача одной строкой

Переписать этот репозиторий (`tri-mcp`) полностью на Rust — как отдельный Rust crate, затем подключить его как vendor в `gHashTag/trios`.

---

## Репозитории

| Репо | Роль |
|---|---|
| `https://github.com/gHashTag/tri-mcp` | **ЗДЕСЬ пишешь Rust код** |
| `https://github.com/gHashTag/trios` | Сюда подключаешь как vendor после готовности |

---

## Что есть сейчас в tri-mcp (TypeScript — УДАЛИТЬ после готовности)

```
browser-tools-server/
  browser-connector.ts   ← Express HTTP сервер порт 3026 + WebSocket ↔ Chrome Extension
  lighthouse/            ← SEO/Performance/Accessibility аудиты (Node.js)
browser-tools-mcp/
  mcp-server.ts          ← MCP stdio сервер (14 tools для Claude/Perplexity)
chrome-extension/        ← НЕ ТРОГАТЬ. Vendor as-is.
```

---

## Что написать на Rust (Ring Architecture)

```
tri-mcp/                             ← корень репо
  Cargo.toml                         ← workspace
  RING.md                            ← обязательно (R3)
  rings/
    SR-00/                           ← 🥈 SILVER: HTTP сервер + WebSocket
      Cargo.toml                     ← name = "trios-mcp-sr00"
      RING.md
      src/
        lib.rs                       ← pub struct BrowserConnector
        server.rs                    ← axum routes (все 19 endpoints)
        auth.rs                      ← Basic Auth middleware
        ws.rs                        ← WebSocket ↔ Chrome Extension
        screenshot.rs                ← capture + save PNG
        logs.rs                      ← LogStore in-memory
    SR-01/                           ← 🥈 SILVER: Lighthouse bridge
      Cargo.toml                     ← name = "trios-mcp-sr01"
      RING.md
      src/
        lib.rs
        lighthouse.rs                ← std::process::Command → node lighthouse
        audits.rs                    ← accessibility/performance/seo/best-practices
    SR-02/                           ← 🥈 SILVER: MCP stdio протокол
      Cargo.toml                     ← name = "trios-mcp-sr02"
      RING.md
      src/
        lib.rs                       ← pub struct McpServer
        protocol.rs                  ← MCP JSON-RPC over stdio
        tools.rs                     ← все 14 tools
        discovery.rs                 ← server discovery порты 3026-3035
    BR-XTASK/                        ← 🥉 BRONZE: launcher
      Cargo.toml                     ← name = "trios-mcp"
      RING.md
      src/
        main.rs                      ← запускает SR-00 + SR-02 параллельно
```

---

## Cargo.toml workspace (корень tri-mcp)

```toml
[workspace]
members = [
    "rings/SR-00",
    "rings/SR-01",
    "rings/SR-02",
    "rings/BR-XTASK",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
axum = { version = "0.7", features = ["ws"] }
tower-http = { version = "0.5", features = ["cors", "auth"] }
tokio-tungstenite = "0.24"
base64 = "0.22"
uuid = { version = "1", features = ["v4"] }
reqwest = { version = "0.12", features = ["json"] }
```

---

## SR-00 — HTTP Server (порт 3026)

Заменяет `browser-tools-server/browser-connector.ts` (1439 строк Express + ws)

### Все axum routes (точное соответствие TS):

```rust
// GET  /.identity      → {"signature": "mcp-browser-connector-24x7"}
// GET  /.port          → {"port": 3026}
// POST /extension-log  ← Chrome Extension → сервер
// GET  /console-logs
// GET  /console-errors
// GET  /network-errors
// GET  /network-success
// GET  /all-xhr
// GET  /selected-element
// POST /selected-element
// POST /current-url
// GET  /current-url
// POST /wipelogs
// POST /capture-screenshot
// POST /screenshot      ← получает base64 от Extension
// POST /accessibility-audit
// POST /performance-audit
// POST /seo-audit
// POST /best-practices-audit
// WS   /extension-ws   ← Chrome Extension WebSocket
```

### Auth:
- Basic Auth: `AUTH_USERNAME` (default: admin), `AUTH_PASSWORD` (default: "")
- Skip auth для `/.identity` и `/.port`
- `ENABLE_AUTH=false` отключает auth

### WebSocket сообщения от Chrome Extension:
```
console-log, console-error, network-request, page-navigated,
selected-element, current-url-response, screenshot-data, screenshot-error
```

### WebSocket сообщения к Extension:
```
take-screenshot (с UUID requestId), server-shutdown
```

### Screenshot:
- Сохранять в `SCREENSHOT_DIR` или `~/.trios/screenshots/`
- base64 PNG decode → файл

---

## SR-01 — Lighthouse Bridge

Заменяет `browser-tools-server/lighthouse/*.ts`

```rust
// lighthouse.rs
pub async fn run_lighthouse(
    url: &str,
    category: AuditCategory,
) -> anyhow::Result<serde_json::Value> {
    let output = std::process::Command::new("node")
        .args([
            "-e",
            &format!("require('lighthouse')('{}', {{output:'json', onlyCategories:['{}']}}).then(r=>process.stdout.write(r.report))", url, category.as_str())
        ])
        .output()?;
    Ok(serde_json::from_slice(&output.stdout)?)
}
```

4 типа аудита: accessibility, performance, seo, best-practices

---

## SR-02 — MCP stdio сервер

Заменяет `browser-tools-mcp/mcp-server.ts`

### Все 14 MCP tools:

```rust
pub enum McpTool {
    GetConsoleLogs,          // GET  /console-logs
    GetConsoleErrors,        // GET  /console-errors
    GetNetworkErrors,        // GET  /network-errors
    GetNetworkLogs,          // GET  /network-success
    TakeScreenshot,          // POST /capture-screenshot
    GetSelectedElement,      // GET  /selected-element
    WipeLogs,                // POST /wipelogs
    RunAccessibilityAudit,   // POST /accessibility-audit
    RunPerformanceAudit,     // POST /performance-audit
    RunSeoAudit,             // POST /seo-audit
    RunBestPracticesAudit,   // POST /best-practices-audit
    RunDebuggerMode,         // static prompt
    RunAuditMode,            // static prompt
    RunNextJsAudit,          // static prompt
}
```

### MCP JSON-RPC stdio loop:

```rust
// protocol.rs
pub async fn run_stdio(host: &str, port: u16) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut writer = BufWriter::new(tokio::io::stdout());
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 { break; }
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let response = handle_request(trimmed, host, port).await;
        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }
    Ok(())
}
```

### Server discovery:
- Проверять порты 3026–3035 последовательно
- GET `/.identity` → проверить `signature == "mcp-browser-connector-24x7"`

---

## BR-XTASK — Launcher (main.rs)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let host = std::env::var("MCP_HOST").unwrap_or("127.0.0.1".into());
    let port: u16 = std::env::var("MCP_PORT")
        .unwrap_or("3026".into()).parse()?;

    let browser_host = host.clone();
    let h1 = tokio::spawn(async move {
        trios_mcp_sr00::BrowserConnector::new(browser_host, port)
            .run().await
    });
    let h2 = tokio::spawn(async move {
        trios_mcp_sr02::McpServer::new(host, port)
            .run_stdio().await
    });

    tokio::select! {
        _ = h1 => {},
        _ = h2 => {},
        _ = tokio::signal::ctrl_c() => {
            println!("\n Shutting down...");
        }
    }
    Ok(())
}
```

---

## LAWS (обязательно)

- **L1**: NO `.sh` файлов. Только `Cargo.toml` и `Makefile`
- **L2**: PR закрывает issue `Closes #N`
- **L3**: `cargo clippy -- -D warnings` = ноль предупреждений
- **L4**: `cargo test` все тесты зелёные (≥10 тестов)
- **L5**: порт **3026** (не 9005)
- **L6**: graceful `Err` возвраты, никаких `panic!` или `.unwrap()` в продакшн коде
- **L8**: каждый файл = немедленный `git add` + `git commit` + `git push`
- **L11**: никаких phantom imports (use X если X не используется)
- **R2**: каждый ring = отдельный `Cargo.toml`
- **R3**: каждый ring directory = `RING.md`
- **R4**: Silver rings не содержат бинарников, Bronze не содержит бизнес-логику
- **R5**: sharing только через path dependency

---

## PHI LOOP (каждый шаг обязателен)

```
edit spec → seal hash → gen → test → verdict → experience → skill commit → git commit
```

---

## Порядок реализации

1. `Cargo.toml` workspace в корне tri-mcp
2. `RING.md` в корне
3. SR-00: `Cargo.toml` → `logs.rs` → `auth.rs` → `screenshot.rs` → `ws.rs` → `server.rs` → `lib.rs` → тесты
4. SR-01: `Cargo.toml` → `lighthouse.rs` → `audits.rs` → `lib.rs` → тесты
5. SR-02: `Cargo.toml` → `discovery.rs` → `tools.rs` → `protocol.rs` → `lib.rs` → тесты
6. BR-XTASK: `Cargo.toml` → `main.rs`
7. `cargo build` → `cargo clippy -- -D warnings` → `cargo test`
8. После зелёного build — добавить в `gHashTag/trios` как vendor (см. ниже)

---

## Проверки после каждого ring (НЕ пропускать)

```bash
# После SR-00:
cargo build -p trios-mcp-sr00 2>&1 | head -20
cargo clippy -p trios-mcp-sr00 -- -D warnings
cargo test -p trios-mcp-sr00

# После SR-01:
cargo build -p trios-mcp-sr01 2>&1 | head -20

# После SR-02:
cargo build -p trios-mcp-sr02 2>&1 | head -20

# Финальный:
cargo build 2>&1 | tail -5
cargo test 2>&1 | tail -20
curl http://127.0.0.1:3026/.identity
```

---

## Подключение в trios как vendor

После зелёного `cargo build` в tri-mcp:

### Шаг 1: добавить в `gHashTag/trios/Cargo.toml`

```toml
[workspace]
members = [
    # ... existing ...
    # trios-mcp vendor (from tri-mcp)
    "vendor/trios-mcp/rings/SR-00",
    "vendor/trios-mcp/rings/SR-01",
    "vendor/trios-mcp/rings/SR-02",
    "vendor/trios-mcp/rings/BR-XTASK",
]
```

### Шаг 2: скопировать rings в trios

```bash
cp -r /Users/playra/tri-mcp/rings /Users/playra/trios/vendor/trios-mcp/
```

### Шаг 3: открыть PR в trios

```
feat(vendor): add trios-mcp Rust crate (replaces tri-mcp TypeScript)
Closes #259
```

---

## DONE checklist

- [ ] `cargo build` в tri-mcp зелёный
- [ ] `cargo clippy -- -D warnings` ноль предупреждений
- [ ] `cargo test` ≥10 тестов зелёных
- [ ] `curl http://127.0.0.1:3026/.identity` возвращает `mcp-browser-connector-24x7`
- [ ] WebSocket `/extension-ws` принимает соединения
- [ ] Все 14 MCP tools отвечают через stdio
- [ ] `RING.md` в каждой ring-директории
- [ ] Отдельный `Cargo.toml` в каждом ring
- [ ] Нет `.sh` файлов (L1)
- [ ] Нет `.unwrap()` в продакшн коде (L6)
- [ ] vendor скопирован в trios
- [ ] PR в trios с `Closes #259`
