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
- [ ] Установить переменные окружения:
  ```bash
  export AUTH_USERNAME=perplexity
  export AUTH_PASSWORD=test123
  ```

### Шаг 2 — Запуск сервера
- [ ] Запустить `browser-tools-server` с аутентификацией:
  ```bash
  AUTH_USERNAME=perplexity AUTH_PASSWORD=test123 npx @agentdeskai/browser-tools-server@latest
  ```
- [ ] Убедиться в логах: `Authentication enabled: username=perplexity`

### Шаг 3 — Настройка Chrome расширения
- [ ] Открыть Chrome DevTools (F12) → вкладка `BrowserToolsMCP`
- [ ] В `Server Connection Settings`:
  - `Enable Authentication` ☑️
  - `Username`: `perplexity`
  - `Password`: `test123`
- [ ] Нажать `Save Settings`
- [ ] Статус: `Connected` ✅

### Шаг 4 — Запуск Tailscale Funnel
- [ ] Запустить туннель (App Store версия, не Homebrew!):
  ```bash
  /Applications/Tailscale.app/Contents/MacOS/Tailscale funnel --https=443
  ```
- [ ] Записать полученный URL туннеля

### Шаг 5 — Верификация туннеля
- [ ] Проверить без auth:
  ```bash
  curl https://<YOUR_TAILSCALE_URL>/.identity
  ```
- [ ] Проверить с auth:
  ```bash
  curl -u perplexity:test123 https://<YOUR_TAILSCALE_URL>/api/status
  ```

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

После выполнения — **создай PR с изменениями этого файла**, отметив результаты каждого шага:
1. Что сработало ✅
2. Что не сработало ❌ + логи ошибок
3. URL туннеля (без пароля в явном виде)
4. Скриншот статуса Chrome расширения

---

> **Генерал ждёт доклада. Удачи, солдат. 🫡**
