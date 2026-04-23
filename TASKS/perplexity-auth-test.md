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
- [x] Установить переменные окружения

### Шаг 2 — Запуск сервера
- [x] Запущен на порту 3025 — `AUTH_USERNAME=perplexity AUTH_PASSWORD=test123`
- [x] Лог подтверждает: `Authentication enabled: username=perplexity`

### Шаг 3 — Настройка Chrome расширения
- [ ] Chrome DevTools (F12) → `BrowserToolsMCP` → `Enable Authentication` ☑️
- [ ] Username: `perplexity`, Password: `test123`
- [ ] `Save Settings` → статус `Connected`
- **⚠️ РУЧНОЙ ШАГ — ждёт человека**

### Шаг 4 — Tailscale Funnel
- [x] Активен: `https://playras-macbook-pro-1.tail01804b.ts.net`

### Шаг 5 — Верификация auth
- [x] Без auth → `401 Authentication required`
- [x] Верные креды → `200`
- [x] Неверные креды → `403 Invalid credentials`

### Шаг 6 — MCP конфигурация
- [x] Готова:
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

### Шаг 7 — Боевые тесты через Perplexity
- [ ] `"Take a screenshot"`
- [ ] `"Get console logs"`
- [ ] `"Run accessibility audit"`
- **⚠️ ОЖИДАЕТ — подключить Perplexity к конфигу выше**

---

## 📊 ИТОГ АВТОТЕСТОВ

| Компонент | Статус | Детали |
|-----------|--------|--------|
| Сервер | ✅ | Порт 3025, auth включён |
| Tailscale Funnel | ✅ | `playras-macbook-pro-1.tail01804b.ts.net` |
| Auth (без кредов) | ✅ | 401 |
| Auth (верные креды) | ✅ | 200 |
| Auth (неверные) | ✅ | 403 |
| Chrome Extension | ⏳ | Ручной шаг |
| Perplexity WebSocket | ⏳ | Ожидает подключения |

---

> **Генерал доволен. Следующий шаг — ручное подключение Chrome Extension и финальный тест через Perplexity. 🫡**
