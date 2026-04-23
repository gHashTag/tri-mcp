# 🎯 БОЕВОЙ ПРИКАЗ от Генерала (Perplexity AI)
> **Солдат — это твоя задача. Выполни точно по шагам. Отчитайся в комментарии к этому файлу через PR.**

---

## 🧠 Контекст

Тестируем систему аутентификации `browser-tools-mcp` с Perplexity агентом, подключаемым через **Tailscale Funnel** туннель.

**Архитектура связи:**
```
Perplexity Agent (WS)
  → wss:// + Basic Auth headers
    → Tailscale Funnel (HTTPS :443)
      → browser-tools-server (localhost:3025)
        → Chrome Extension (WebSocket)
```

**Детали аутентификации:**
- `AUTH_USERNAME=perplexity`
- `AUTH_PASSWORD=test123`
- Base64: `cGVycGxleGl0eTp0ZXN0MTIz`

---

## ✅ ЧЕКЛИСТ ЗАДАЧ

### Шаг 1 — Настройка кредов
- [x] Установить переменные окружения:
  ```bash
  export AUTH_USERNAME=perplexity
  export AUTH_PASSWORD=test123
  ```

### Шаг 2 — Запуск сервера
- [x] Запустить `browser-tools-server` с аутентификацией:
  ```bash
  AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx @agentdeskai/browser-tools-server@latest
  ```
- [x] Убедиться в логах: `Authentication enabled: username=perplexity`

### Шаг 3 — Настройка Chrome расширения
- [ ] Открыть Chrome DevTools (F12) → вкладка `BrowserToolsMCP`
- [ ] В `Server Connection Settings`:
  - `Enable Authentication` ☑️
  - `Username`: `perplexity`
  - `Password`: `test123`
- [ ] Нажать `Save Settings`
- [ ] Статус: `Connected` ✅

### Шаг 4 — Запуск Tailscale Funnel
- [x] Запустить туннель (App Store версия, не Homebrew!):
  ```bash
  /Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443
  ```
- [x] Записать полученный URL туннеля: `https://playras-macbook-pro-1.tail01804b.ts.net`

### Шаг 5 — Верификация туннеля
- [x] Проверить без auth:
  ```bash
  curl https://<YOUR_TAILSCALE_URL>/.identity
  ```
- [x] Проверить с auth:
  ```bash
  curl -u perplexity:test123 https://<YOUR_TAILSCALE_URL>/api/status
  ```

#### Результаты тестов curl (2026-04-23):

| Тест | Учетные данные | HTTP код | Результат |
|------|----------------|-----------|-----------|
| /.identity (без auth) | - | 200 | ✅ эндпоинт открыт по дизайну |
| /.identity (с auth) | perplexity:test123 | 200 | ✅ |
| /console-logs (без auth) | - | 401 | ✅ Authentication required |
| /console-logs (с правильными кредами) | perplexity:test123 | 200 | ✅ |
| /console-logs (с неправильными кредами) | wrong:creds | 403 | ✅ Invalid credentials |

**Вывод:** Аутентификация работает корректно!

### Шаг 6 — MCP конфигурация для Perplexity
- [ ] Использовать конфигурацию:
  ```json
  {
    "mcpServers": [{
      "url": "wss://<YOUR_TAILSCALE_URL>",
      "transport": "websocket",
      "headers": {
        "Authorization": "Basic cGVycGxleGl0eTp0ZXN0MTIz"
      }
    }]
  }
  ```

### Шаг 7 — Боевые тесты через Perplexity
- [ ] `"Take a screenshot"` → скриншот текущей вкладки
- [ ] `"Get console logs"` → консольные логи браузера
- [ ] `"Run accessibility audit"` → аудит доступности

---

## 🔍 Ключевые файлы для проверки

| Файл | Зона ответственности |
|------|---------------------|
| `browser-tools-server/browser-connector.ts` | Логика аутентификации (строки 247-281) |
| `chrome-extension/panel.js` | UI настроек авторизации |
| `chrome-extension/devtools.js` | WebSocket соединение |

---

## 🛠 Дебаг-команды если что-то не работает

```bash
# Туннель живой?
curl https://<TAILSCALE_URL>/.identity

# Сервер принимает auth?
curl -u perplexity:test123 http://localhost:3025/.identity

# WebSocket сервер отвечает?
npm install -g wscat
wscat -c ws://localhost:3025/extension-ws
```

---

## 📋 Отчёт солдата

### Что сработало ✅

1. **Настройка кредов** - переменные окружения установлены
2. **Запуск сервера** - сервер запущен на порту 3025 с `AUTH_USERNAME=perplexity`, `AUTH_PASSWORD=test123`
3. **Tailscale Funnel** - туннель активен и проксирует на порт 3025
4. **Верификация туннеля** - все curl тесты пройдены:
   - Без auth: 401 (Authentication required)
   - С правильными кредами: 200
   - С неправильными кредами: 403 (Invalid credentials)

### Что не сработало ❌

- Ничего! Все автоматизированные тесты пройдены успешно.

### URL туннеля

`https://playras-macbook-pro-1.tail01804b.ts.net`

### MCP конфигурация для Perplexity

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

### Осталось выполнить

- [ ] Шаг 3 — Настройка Chrome расширения (требуется ручная настройка в DevTools)
- [ ] Шаг 6 — Подключение Perplexity агента
- [ ] Шаг 7 — Боевые тесты через Perplexity

### Дата и статус

- **Дата:** 2026-04-23
- **Исполнитель:** Claude (browser-tools-mcp)
- **Версия сервера:** 1.2.0
- **Статус:** Аутентификация протестирована, туннель активен, ожидает ручной настройки Chrome расширения и подключения Perplexity агента

---

> **Генерал ждёт доклада. Удачи, солдат. 🫡**
