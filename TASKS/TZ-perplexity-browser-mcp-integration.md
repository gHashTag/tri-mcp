# ТЗ: Интеграция Perplexity Agent с browser-tools-mcp через Tailscale Funnel

## Цель

Обеспечить полноценное подключение Perplexity агента к локальному `browser-tools-server` через защищённый WebSocket-туннель (Tailscale Funnel) с Basic Auth. После выполнения задачи агент должен уметь делать скриншоты, читать консоль браузера и запускать accessibility аудиты через MCP-инструменты.

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

## Scope работы

### Задача 1: Автоматический запуск сервера

**Проблема:** Сейчас сервер запускается вручную с env-переменными.  
**Решение:** Создать `start-browser-mcp.sh` скрипт и/или `launchd` plist для автозапуска на macOS.

**Критерии выполнения:**
- [ ] Файл `scripts/start-browser-mcp.sh` — запускает сервер с корректными `AUTH_USERNAME` / `AUTH_PASSWORD`
- [ ] Файл `scripts/start-tunnel.sh` — запускает Tailscale Funnel на порт 3025
- [ ] Оба скрипта идемпотентны (повторный запуск не падает)
- [ ] Опционально: `com.woodypecker.browser-mcp.plist` для автозапуска через `launchd`

**Команды:**
```bash
# start-browser-mcp.sh
pkill -f "browser-tools-server" 2>/dev/null || true
AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 \
  npx @agentdeskai/browser-tools-server@latest &

# start-tunnel.sh
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel reset 2>/dev/null || true
/Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443 3025 --bg
```

---

### Задача 2: Хранение credentials в .env

**Проблема:** Пароли хардкодятся в командах и документации.  
**Решение:** Вынести в `.env` файл, добавить `.env.example`.

**Критерии выполнения:**
- [ ] Файл `.env.example` с шаблоном:
  ```
  AUTH_USERNAME=perplexity
  AUTH_PASSWORD=test123
  TAILSCALE_URL=https://playras-macbook-pro-1.tail01804b.ts.net
  ```
- [ ] `.env` добавлен в `.gitignore`
- [ ] Скрипты из Задачи 1 читают переменные через `source .env`

---

### Задача 3: Настройка Chrome Extension

**Проблема:** Chrome extension требует ручной настройки через DevTools UI.  
**Решение:** Задокументировать процесс и добавить скриншоты в README.

**Критерии выполнения:**
- [ ] В `README.md` (или отдельном `SETUP.md`) есть раздел "Chrome Extension Setup" с шагами:
  1. Открыть DevTools → BrowserToolsMCP tab
  2. Установить `localhost:3025`
  3. Включить Authentication с credentials из `.env`
  4. Проверить статус `Connected ✅`
- [ ] Скриншот или ascii-схема настройки прилагается

---

### Задача 4: MCP конфигурация для Perplexity

**Проблема:** Конфигурация подключения нигде не зафиксирована в репозитории.  
**Решение:** Добавить готовый JSON-конфиг.

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
- [ ] В документации указано как обновить Base64 при смене пароля:
  ```bash
  echo -n "username:password" | base64
  ```

---

### Задача 5: Health-check и smoke-тесты

**Проблема:** Нет автоматической проверки работоспособности стека.  
**Решение:** Скрипт `scripts/health-check.sh` с набором curl-тестов.

**Критерии выполнения:**
- [ ] Скрипт проверяет все 4 сценария:

  | Тест | Endpoint | Ожидаемый HTTP код |
  |------|----------|--------------------|
  | Public endpoint | `/.identity` | 200 |
  | No auth | `/console-logs` | 401 |
  | Valid auth | `/console-logs` | 200 |
  | Wrong auth | `/console-logs` | 403 |

- [ ] Скрипт выводит ✅ / ❌ для каждого теста
- [ ] Скрипт возвращает exit code `1` если хотя бы один тест провалился

```bash
#!/bin/bash
source "$(dirname "$0")/../.env" 2>/dev/null || true
BASE_URL="${TAILSCALE_URL:-https://playras-macbook-pro-1.tail01804b.ts.net}"
PASS=0; FAIL=0

check() {
  local desc=$1 url=$2 expected=$3; shift 3
  local code; code=$(curl -s -o /dev/null -w "%{http_code}" "$@" "$url")
  if [ "$code" = "$expected" ]; then echo "✅ $desc → $code"; ((PASS++))
  else echo "❌ $desc → $code (expected $expected)"; ((FAIL++)); fi
}

check "Public /.identity"     "$BASE_URL/.identity"    200
check "No auth /console-logs" "$BASE_URL/console-logs" 401
check "Valid auth"            "$BASE_URL/console-logs" 200 -u "$AUTH_USERNAME:$AUTH_PASSWORD"
check "Wrong auth"            "$BASE_URL/console-logs" 403 -u "wrong:creds"

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ $FAIL -eq 0 ] && exit 0 || exit 1
```

---

### Задача 6: Финальное end-to-end тестирование с Perplexity

**Проблема:** Статус подключения Perplexity — `⏳ Pending`.  
**Решение:** Провести ручное E2E тестирование и обновить статус в `POLICY-perplexity-auth-setup.md`.

**Критерии выполнения:**
- [ ] Perplexity агент успешно подключается через WSS
- [ ] Команда **"Take a screenshot"** — возвращает скриншот текущего таба
- [ ] Команда **"Get console logs"** — возвращает лог браузера
- [ ] Команда **"Run accessibility audit"** — возвращает WCAG-отчёт
- [ ] Статус в `POLICY-perplexity-auth-setup.md` обновлён:
  | Component | Status |
  |---|---|
  | Perplexity Connection | ✅ Connected |
  | Chrome Extension | ✅ Configured |
  | Tailscale Funnel | ✅ Running |

---

## Технические ограничения

- **Tailscale версия**: обязательно App Store (`/Applications/Tailscale.app`), не Homebrew — Homebrew версия не поддерживает WebSocket в Funnel
- **Порт**: `3025` (browser-tools-server), `443` (Funnel HTTPS)
- **Протокол**: WebSocket (`wss://`), не HTTP
- **Auth**: HTTP Basic Auth через заголовок `Authorization: Basic <base64>`
- **Credentials**: хранить только в `.env`, не в коде

---

## Приоритет задач

| # | Задача | Приоритет | Оценка |
|---|--------|-----------|--------|
| 1 | Автозапуск сервера | 🔴 High | 1h |
| 2 | .env файл | 🔴 High | 30m |
| 3 | Chrome Extension docs | 🟡 Medium | 1h |
| 4 | MCP конфиг JSON | 🔴 High | 30m |
| 5 | Health-check скрипт | 🟡 Medium | 1h |
| 6 | E2E тест с Perplexity | 🔴 High | 2h |

**Итого:** ~6 часов

---

## Структура файлов (создать)

```
tri-mcp/
├── .env.example                          # шаблон переменных окружения
├── config/
│   └── mcp-perplexity.json               # MCP конфиг для Perplexity
└── scripts/
    ├── start-browser-mcp.sh              # запуск browser-tools-server
    ├── start-tunnel.sh                   # запуск Tailscale Funnel
    └── health-check.sh                   # smoke-тесты
```

---

## Связанные файлы

| Файл | Описание |
|------|----------|
| `TASKS/POLICY-perplexity-auth-setup.md` | Исходный гайд по настройке |
| `scripts/start-browser-mcp.sh` | Скрипт запуска сервера (создать) |
| `scripts/start-tunnel.sh` | Скрипт запуска туннеля (создать) |
| `scripts/health-check.sh` | Smoke-тесты (создать) |
| `config/mcp-perplexity.json` | MCP конфиг для Perplexity (создать) |
| `.env.example` | Шаблон переменных окружения (создать) |

---

**Создано:** 2026-04-27  
**Источник:** `TASKS/POLICY-perplexity-auth-setup.md`  
**Версия:** 1.0.0
