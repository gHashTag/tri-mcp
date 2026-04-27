# ПЛАН ДЛЯ АГЕНТА C — Rust Rewrite tri-mcp

**Репозиторий:** [gHashTag/tri-mcp](https://github.com/gHashTag/tri-mcp)
**Цель:** Полная замена TypeScript → Pure Rust (Ring Architecture)
**Эталон:** trios-a2a (8 SR-rings + BR-OUTPUT, 84 tests)

---

## КОНТЕКСТ: Что переписываем

Из реального исходного кода [`browser-connector.ts`](https://github.com/gHashTag/tri-mcp/blob/42e8fdf0a237f3d30ba162f681430f5c570e6249/browser-tools-server/browser-connector.ts) (51 KB) и [`mcp-server.ts`](https://github.com/gHashTag/tri-mcp/blob/42e8fdf0a237f3d30ba162f681430f5c570e6249/browser-tools-mcp/mcp-server.ts) (49 KB):

| TS компонент | Что делает | Rust замена |
|---|---|---|
| `express` + `cors` + `bodyParser` | HTTP сервер | `axum 0.7` в SR-02 |
| `WebSocketServer` + `/extension-ws` | WS от Chrome Extension | `axum::extract::ws` в SR-02 |
| `consoleLogs[]`, `networkErrors[]` и др. | In-memory storage | `RwLock<Vec<T>>` в AppState, SR-02 |
| `basicAuthMiddleware` | Basic Auth | `axum::middleware` в SR-02 |
| `/.identity`, `/.port`, `/health` | Служебные endpoints | Routes в SR-02 |
| 4x `setupAuditEndpoint` | Lighthouse audits | `SR-03` через `chromiumoxide` |
| `captureScreenshot` | Screenshot via WS | `SR-03` |
| `REQUESTED_PORT = 3025` ❌ | Дефолтный порт | `ServerPort(3026)` в SR-00 |
| Tailscale funnel | Публичный URL | `SR-04` |
| `main()` entry point | Запуск сервера | `BR-XTASK` |

---

## ПРАВИЛА (ЗАКОНЫ — НАРУШЕНИЕ = БЛОКИРОВКА)

```
LAW-1: cargo build --workspace exits 0 (ВСЕГДА)
LAW-2: cargo test --workspace exits 0 (ВСЕГДА)
LAW-3: cargo clippy --workspace -- -D warnings exits 0 (ВСЕГДА)
LAW-4: SR-N импортирует только SR-(N-1) и ниже (нет циклов)
LAW-5: SR-00 — zero dependencies (никаких imports из rings/*)
LAW-6: Pure Rust only в rings/ (find rings/ -name "*.ts" → пусто)

AXUM-LAW: НИКОГДА Extension<u16/i64/String> — только newtype!
  ❌ Extension<u16> → компиляция сломается
  ✅ Extension<ServerPort> → работает
  ✅ State<Arc<AppState>> → предпочтительно
```

---

## СТРУКТУРА

```
tri-mcp/
├── Cargo.toml          ← workspace (уже есть)
├── src/lib.rs          ← re-export всех rings (уже есть)
├── RING.md             ← manifest
└── rings/
    ├── SR-00/          ← ✅ ГОТОВ (Identity & Config)
    ├── SR-01/          ← ✅ ГОТОВ (Protocol Types)
    ├── SR-02/          ← ⚠️  СКЕЛЕТ (Transport — главная работа)
    ├── SR-03/          ← ❌ НЕ СОЗДАН (14 MCP Tools)
    ├── SR-04/          ← ✅ СОЗДАН (Tailscale Tunnel)
    └── BR-XTASK/       ← ✅ СКЕЛЕТ (Launcher)
```

---

## BATCH 1 — Дополнить SR-00 (30 мин)

SR-00 уже собирается. Убедиться что есть **все типы** из реального TS кода:

```rust
// rings/SR-00/src/lib.rs

#[derive(Debug, Clone, Copy)]
pub struct ServerPort(pub u16);

impl Default for ServerPort {
    fn default() -> Self { Self(3026) }  // ← 3026, НЕ 3025!
}

#[derive(Debug, Clone)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    pub enabled: bool,   // ← ENABLE_AUTH из TS
}

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub port: ServerPort,
    pub auth: AuthCredentials,
    pub server_host: String,     // SERVER_HOST env
    pub log_limit: usize,        // logLimit: 50
    pub query_limit: usize,      // queryLimit: 30000
    pub string_size_limit: usize,// stringSizeLimit: 500
    pub max_log_size: usize,     // maxLogSize: 20000
    pub screenshot_path: String, // screenshotPath
}

impl McpConfig {
    pub fn from_env() -> Self { /* читаем env vars */ }
}
```

**Тесты SR-00:**
```rust
#[test] fn test_default_port_is_3026()
#[test] fn test_auth_validate_base64()
#[test] fn test_config_from_env()
#[test] fn test_server_host_default()
```

```bash
cargo test -p tri-mcp-sr00
cargo clippy -p tri-mcp-sr00 -- -D warnings
```

---

## BATCH 2 — Дополнить SR-01 (1 час)

Типы из реального TS кода (`browser-connector.ts` in-memory arrays + `mcp-server.ts` protocol):

```rust
// rings/SR-01/src/lib.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserLog {
    pub r#type: String,   // "console-log" | "console-error"
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub status: u16,
    pub timestamp: String,
    pub request_headers: Option<serde_json::Value>,
    pub response_headers: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedElement {
    pub tag_name: String,
    pub id: Option<String>,
    pub class_name: Option<String>,
    pub attributes: serde_json::Value,
}

// WS сообщения от Chrome Extension (из browser-connector.ts wss.on("message"))
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtensionMessage {
    #[serde(rename = "screenshot-data")]
    ScreenshotData { data: String, path: Option<String>, auto_paste: Option<bool> },
    #[serde(rename = "screenshot-error")]
    ScreenshotError { error: String },
    #[serde(rename = "current-url-response")]
    UrlResponse { url: String, tab_id: Option<serde_json::Value>, request_id: Option<String> },
    #[serde(rename = "page-navigated")]
    PageNavigated { url: String, tab_id: Option<serde_json::Value> },
}

// MCP Tool definition (для SR-03)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}
```

```bash
cargo test -p tri-mcp-sr01
cargo clippy -p tri-mcp-sr01 -- -D warnings
```

---

## BATCH 3 — SR-02: Transport Layer (главный, 5 часов)

Это полная замена `browser-connector.ts`. Все 19 routes из TS кода:

```rust
// rings/SR-02/src/lib.rs

pub struct AppState {
    pub config: McpConfig,                              // SR-00
    pub console_logs: RwLock<Vec<BrowserLog>>,          // SR-01
    pub console_errors: RwLock<Vec<BrowserLog>>,
    pub network_errors: RwLock<Vec<NetworkRequest>>,
    pub network_success: RwLock<Vec<NetworkRequest>>,
    pub selected_element: RwLock<Option<SelectedElement>>,
    pub current_url: RwLock<String>,
    pub current_tab_id: RwLock<Option<serde_json::Value>>,
    pub screenshot_callbacks: RwLock<HashMap<String, oneshot::Sender<ScreenshotResult>>>,
    pub ws_tx: RwLock<Option<SplitSink<WebSocket, Message>>>, // активное WS соединение
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Публичные (без auth) — из TS: req.path === '/.identity' || req.path === '/.port'
        .route("/.identity", get(identity_handler))
        .route("/.port",     get(port_handler))
        // Auth-protected
        .route("/extension-log",      post(extension_log_handler))
        .route("/console-logs",       get(get_console_logs))
        .route("/console-errors",     get(get_console_errors))
        .route("/network-errors",     get(get_network_errors))
        .route("/network-success",    get(get_network_success))
        .route("/all-xhr",            get(get_all_xhr))
        .route("/selected-element",   get(get_selected_element).post(post_selected_element))
        .route("/current-url",        get(get_current_url).post(post_current_url))
        .route("/wipelogs",           post(wipe_logs))
        .route("/screenshot",         post(store_screenshot))
        .route("/capture-screenshot", post(capture_screenshot))
        .route("/accessibility-audit",  post(accessibility_audit))
        .route("/performance-audit",    post(performance_audit))
        .route("/seo-audit",            post(seo_audit))
        .route("/best-practices-audit", post(best_practices_audit))
        .route("/extension-ws",       get(ws_handler))   // WebSocket upgrade
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}
```

**Auth middleware — ПРАВИЛЬНО для Axum 0.7:**
```rust
async fn auth_middleware(
    State(state): State<Arc<AppState>>,   // ✅ State<Arc<T>>
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    // Пропускаем /.identity и /.port без auth
    if path == "/.identity" || path == "/.port" {
        return next.run(request).await;
    }
    if !state.config.auth.enabled {
        return next.run(request).await;
    }
    // Проверяем Basic Auth header
    match request.headers().get("authorization") {
        None => Response::builder()
            .status(401)
            .header("WWW-Authenticate", "Basic realm=\"Browser Tools Server\"")
            .body(Body::empty()).unwrap(),
        Some(header) => {
            // base64 decode и проверка username:password
            if validate_basic_auth(header, &state.config.auth) {
                next.run(request).await
            } else {
                Response::builder().status(403).body(Body::empty()).unwrap()
            }
        }
    }
}
```

**WebSocket handler (замена `wss.on("connection")`):**
```rust
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    *state.ws_tx.write().await = Some(sender);

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let event: ExtensionMessage = serde_json::from_str(&text)?;
            match event {
                ExtensionMessage::ScreenshotData { data, path, auto_paste } => {
                    // Resolve pending screenshot callback
                }
                ExtensionMessage::PageNavigated { url, tab_id } => {
                    *state.current_url.write().await = url;
                }
                // ... остальные типы
            }
        }
    }
}
```

**Тесты SR-02 (минимум 10):**
```rust
#[tokio::test] async fn test_identity_endpoint_no_auth()
#[tokio::test] async fn test_port_endpoint_no_auth()
#[tokio::test] async fn test_console_logs_requires_auth()
#[tokio::test] async fn test_console_logs_valid_auth()
#[tokio::test] async fn test_console_logs_wrong_auth()
#[tokio::test] async fn test_extension_log_stores_console()
#[tokio::test] async fn test_extension_log_stores_network()
#[tokio::test] async fn test_wipe_logs()
#[tokio::test] async fn test_all_xhr_merges_and_sorts()
#[tokio::test] async fn test_ws_upgrade_succeeds()
```

```bash
cargo test -p tri-mcp-sr02
cargo clippy -p tri-mcp-sr02 -- -D warnings
```

---

## BATCH 4 — SR-03: 14 MCP Tools (4 часа)

Замена `mcp-server.ts` (49 KB) + Lighthouse calls из `puppeteer-service.ts`:

```rust
// rings/SR-03/src/lib.rs

pub fn all_tools() -> Vec<McpTool> {
    vec![
        tool("getScreenshot",          "Capture current tab screenshot"),
        tool("getConsoleLogs",         "Get browser console output"),
        tool("getConsoleErrors",       "Get console errors only"),
        tool("getNetworkRequests",     "Get all XHR/fetch requests"),
        tool("getNetworkErrors",       "Get failed network requests"),
        tool("getDomContent",          "Get full page HTML"),
        tool("getSelectedElement",     "Get selected DOM element"),
        tool("getAccessibilityTree",   "Get accessibility tree"),
        tool("wipeLogs",               "Clear all stored logs"),
        tool("runAccessibilityAudit",  "Run Lighthouse accessibility audit"),
        tool("runPerformanceAudit",    "Run Lighthouse performance audit"),
        tool("runSeoAudit",            "Run Lighthouse SEO audit"),
        tool("runBestPracticesAudit",  "Run Lighthouse best practices audit"),
        tool("runNextjsAudit",         "Run Next.js specific audit"),
    ]
}

// Вызов инструмента — делегирует в SR-02 AppState
pub async fn call_tool(
    name: &str,
    _args: serde_json::Value,
    state: Arc<AppState>,   // SR-02
) -> ToolResult { ... }
```

**Важно:** Lighthouse audits вызываются через `tokio::process::Command` (`npx lighthouse`) или через `chromiumoxide`. Простейший вариант — shell-out к существующему `npx lighthouse`, как делает TS версия.

```bash
cargo test -p tri-mcp-sr03
cargo clippy -p tri-mcp-sr03 -- -D warnings
```

---

## BATCH 4b — SR-04: Tailscale Tunnel (30 мин)

Уже создан, дополнить реализацию:

```rust
// rings/SR-04/src/lib.rs
pub async fn start_funnel(port: ServerPort) -> Result<String, String> {
    let output = tokio::process::Command::new(
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale"
    )
    .args(["funnel", "--https=443", "--bg", &port.0.to_string()])
    .output()
    .await
    .map_err(|e| e.to_string())?;

    if output.status.success() {
        // Парсим URL из stdout
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines()
            .find(|l| l.contains("https://"))
            .unwrap_or("Funnel started")
            .to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
```

---

## BATCH 5 — BR-XTASK: Entry Point (1 час)

```rust
// rings/BR-XTASK/src/main.rs
use tri_mcp_sr00::McpConfig;
use tri_mcp_sr02::{AppState, build_router};
use tri_mcp_sr04::start_funnel;

#[tokio::main]
async fn main() {
    tracing_subscriber::init();

    let config = McpConfig::from_env();
    let port = config.port;
    let host = config.server_host.clone();

    let state = Arc::new(AppState::new(config));

    // Опциональный Tailscale Funnel
    if std::env::var("TAILSCALE_FUNNEL").is_ok() {
        match start_funnel(port).await {
            Ok(url) => tracing::info!("Public URL: {}", url),
            Err(e)  => tracing::warn!("Funnel failed: {}", e),
        }
    }

    let app = build_router(state);
    let addr = format!("{}:{}", host, port.0);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("=== Browser Tools Server Started ===");
    tracing::info!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
```

---

## ФИНАЛЬНЫЕ ПРОВЕРКИ

```bash
# После каждого batch:
cargo check -p tri-mcp-sr0X
cargo test -p tri-mcp-sr0X
cargo clippy -p tri-mcp-sr0X -- -D warnings

# После BR-XTASK:
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

# Функциональный тест:
PORT=3026 AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 \
  cargo run -p tri-mcp-br-xtask

curl http://127.0.0.1:3026/.identity
# → {"signature":"mcp-browser-connector-24x7","port":3026}

cargo run -p health-check
# → 4/4 ✅
```

---

## КРИТЕРИЙ ЗАВЕРШЕНИЯ

```
✅ cargo build --workspace → exit 0
✅ cargo test --workspace  → все тесты green
✅ cargo clippy --workspace -- -D warnings → 0 warnings
✅ curl /.identity → {"signature":"mcp-browser-connector-24x7"}
✅ health-check → 4/4
✅ Chrome Extension подключается по WS
✅ find browser-tools-server browser-tools-mcp -name "*.ts"
   → можно удалить (TypeScript мёртв)
```
