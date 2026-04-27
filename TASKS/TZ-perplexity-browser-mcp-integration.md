# ТЗ: Интеграция Perplexity Agent с browser-tools-mcp через Tailscale Funnel

## Цель

Обеспечить полноценное подключение Perplexity агента к локальному `browser-tools-server` через защищённый WebSocket-туннель (Tailscale Funnel) с Basic Auth.

**Стек:** Только Rust. Все утилиты — Rust binary crates. `.sh` скрипты запрещены.

После выполнения задачи агент должен уметь делать скриншоты, читать консоль браузера и запускать accessibility аудиты через MCP-инструменты.

---

## Архитектура

```
Perplexity Agent (WSS client)
  → wss:// + Basic Auth header
    → Tailscale Funnel (HTTPS :443)
      → browser-tools-server (localhost:3025)
        → Chrome Extension (WebSocket)
          → Chrome DevTools
```

---

## Структура crate

```
tri-mcp/
├── Cargo.toml                          # workspace
├── crates/
│   ├── browser-mcp-launcher/           # Задача 1: запуск сервера
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── tunnel-launcher/                # Задача 1: запуск Tailscale Funnel
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── health-check/                   # Задача 5: smoke-тесты
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── config/
│   └── mcp-perplexity.json             # Задача 4: MCP конфиг
└── .env.example                        # Задача 2: шаблон env
```

---

## Scope работ

### Задача 1: Crate `browser-mcp-launcher`

**Проблема:** Сервер и туннель запускаются вручную.  
**Решение:** Два Rust binary crate — `browser-mcp-launcher` и `tunnel-launcher`.

**Зависимости:**
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"
anyhow = "1"
```

**Критерии выполнения:**

`crates/browser-mcp-launcher/src/main.rs`:
- [ ] Читает `AUTH_USERNAME` и `AUTH_PASSWORD` из `.env` через `dotenvy`
- [ ] Убивает существующий процесс `browser-tools-server` если запущен (`sysinfo` crate)
- [ ] Запускает `npx @agentdeskai/browser-tools-server@latest` через `tokio::process::Command` с env-переменными
- [ ] Идемпотентен — повторный запуск не паникует
- [ ] Логирует статус в stdout

```rust
// Пример структуры main.rs
use tokio::process::Command;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let username = env::var("AUTH_USERNAME")?;
    let password = env::var("AUTH_PASSWORD")?;

    Command::new("npx")
        .args(["@agentdeskai/browser-tools-server@latest"])
        .env("AUTH_USERNAME", &username)
        .env("AUTH_PASSWORD", &password)
        .spawn()?
        .wait()
        .await?;
    Ok(())
}
```

`crates/tunnel-launcher/src/main.rs`:
- [ ] Читает `TAILSCALE_URL` из `.env`
- [ ] Сбрасывает старый Funnel через `Command::new("/Applications/Tailscale.app/Contents/MacOS/Tailscale").args(["funnel", "reset"])`
- [ ] Запускает `funnel --https=443 3025 --bg`
- [ ] Выводит итоговый URL туннеля

---

### Задача 2: `.env.example`

**Критерии выполнения:**
- [ ] Файл `.env.example` в корне репозитория:
  ```
  AUTH_USERNAME=perplexity
  AUTH_PASSWORD=test123
  TAILSCALE_URL=https://playras-macbook-pro-1.tail01804b.ts.net
  ```
- [ ] `.env` добавлен в `.gitignore`
- [ ] Все crates читают переменные через `dotenvy::dotenv()`

---

### Задача 3: Документация Chrome Extension

**Критерии выполнения:**
- [ ] В `README.md` или `SETUP.md` раздел "Chrome Extension Setup":
  1. Открыть DevTools → BrowserToolsMCP tab
  2. Host: `localhost`, Port: `3025`
  3. Enable Authentication: `AUTH_USERNAME` / `AUTH_PASSWORD` из `.env`
  4. Статус `Connected ✅`

---

### Задача 4: MCP конфиг для Perplexity

**Критерии выполнения:**
- [ ] Файл `config/mcp-perplexity.json`:
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
- [ ] В README указано как пересгенерировать Base64:
  ```rust
  // Утилита в crate или однострочник через base64 crate
  use base64::{Engine, engine::general_purpose};
  let encoded = general_purpose::STANDARD.encode("username:password");
  ```

---

### Задача 5: Crate `health-check`

**Проблема:** Нет автоматической проверки стека.  
**Решение:** Rust binary crate `health-check` использует `reqwest` для curl-эквивалентных проверок.

**Зависимости:**
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
dotenvy = "0.15"
anyhow = "1"
colored = "2"
```

**Критерии выполнения:**
- [ ] Проверяет 4 сценария:

  | Тест | Endpoint | Ожидаемый HTTP код |
  |------|----------|-------------------|
  | Public endpoint | `/.identity` | 200 |
  | No auth | `/console-logs` | 401 |
  | Valid auth | `/console-logs` | 200 |
  | Wrong auth | `/console-logs` | 403 |

- [ ] Выводит `✅ / ❌` с цветом через `colored` crate
- [ ] Завершается с `process::exit(1)` если хотя бы один тест провалился

```rust
// Пример структуры
use reqwest::Client;
use colored::Colorize;

async fn check(client: &Client, desc: &str, url: &str, expected: u16, auth: Option<(&str, &str)>) -> bool {
    let mut req = client.get(url);
    if let Some((user, pass)) = auth {
        req = req.basic_auth(user, Some(pass));
    }
    let status = req.send().await.unwrap().status().as_u16();
    if status == expected {
        println!("{} {} → {}", "✅".green(), desc, status);
        true
    } else {
        println!("{} {} → {} (expected {})", "❌".red(), desc, status, expected);
        false
    }
}
```

---

### Задача 6: End-to-end тест с Perplexity

**Критерии выполнения:**
- [ ] Запустить `cargo run -p browser-mcp-launcher`
- [ ] Запустить `cargo run -p tunnel-launcher`
- [ ] Запустить `cargo run -p health-check` — все 4 теста ✅
- [ ] Подключить Perplexity через `config/mcp-perplexity.json`
- [ ] Команда **"Take a screenshot"** — возвращает скриншот текущего таба
- [ ] Команда **"Get console logs"** — возвращает лог браузера
- [ ] Команда **"Run accessibility audit"** — возвращает WCAG-отчёт
- [ ] Обновить статус в `POLICY-perplexity-auth-setup.md`:

  | Component | Status |
  |---|---|
  | Perplexity Connection | ✅ Connected |
  | Chrome Extension | ✅ Configured |
  | Tailscale Funnel | ✅ Running |

---

## Технические ограничения

- **Язык:** Только Rust. `.sh` скрипты запрещены
- **Runtime:** `tokio` для async, `dotenvy` для `.env`
- **HTTP клиент:** `reqwest` (не curl, не ureq)
- **Tailscale версия:** App Store (`/Applications/Tailscale.app`), не Homebrew — нет WebSocket поддержки в Funnel у Homebrew версии
- **Порт:** `3025` (browser-tools-server), `443` (Funnel HTTPS)
- **Протокол:** WebSocket (`wss://`), не HTTP
- **Auth:** HTTP Basic Auth → `Authorization: Basic <base64>`
- **Credentials:** только в `.env`, никогда в коде

---

## Приоритет задач

| # | Задача | Приоритет | Оценка |
|---|--------|-----------|--------|
| 1 | Crate `browser-mcp-launcher` + `tunnel-launcher` | 🔴 High | 2h |
| 2 | `.env.example` + `.gitignore` | 🔴 High | 30m |
| 3 | Chrome Extension docs | 🟡 Medium | 1h |
| 4 | `config/mcp-perplexity.json` | 🔴 High | 30m |
| 5 | Crate `health-check` | 🔴 High | 2h |
| 6 | E2E тест с Perplexity | 🔴 High | 2h |

**Итого:** ~8 часов

---

**Создано:** 2026-04-27  
**Источник:** `TASKS/POLICY-perplexity-auth-setup.md`  
**Версия:** 2.0.0 (Rust-only rewrite)
