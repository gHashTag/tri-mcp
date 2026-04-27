#!/usr/bin/env bash
# funnel-smoke.sh — SR-02 P-10 funnel-side verification
# Usage: bash funnel-smoke.sh
# Override: BASE=https://... AUTH_USERNAME=... AUTH_PASSWORD=... bash funnel-smoke.sh

set -euo pipefail

BASE="${BASE:-https://playras-macbook-pro-1.tail01804b.ts.net}"
USER="${AUTH_USERNAME:-perplexity}"
PASS="${AUTH_PASSWORD:-test123}"
AUTH="$(printf '%s:%s' "$USER" "$PASS" | base64)"
MCP="$BASE/mcp"

PASS_COUNT=0
FAIL_COUNT=0

pass() { echo "✅ PASS — $1"; ((PASS_COUNT++)) || true; }
fail() { echo "❌ FAIL — $1"; ((FAIL_COUNT++)) || true; }

echo "======================================="
echo " SR-02 Funnel Smoke — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo " Target : $MCP"
echo " User   : $USER"
echo "======================================="
echo ""

# P-01 TLS reachability (no auth)
echo "[P-01] TLS reachability (no auth header)"
HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$MCP" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{}}' \
  --max-time 10 || echo "000")
if [[ "$HTTP" == "401" ]]; then
  pass "TLS OK, got HTTP 401 (unauthenticated)"
else
  fail "Expected HTTP 401, got HTTP $HTTP"
fi

# P-02 Wrong credentials → 403
echo "[P-02] Wrong credentials"
HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$MCP" \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $(printf 'perplexity:wrongpassword' | base64)" \
  -d '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{}}' \
  --max-time 10 || echo "000")
if [[ "$HTTP" == "403" ]]; then
  pass "Got HTTP 403 for wrong creds"
else
  fail "Expected HTTP 403, got HTTP $HTTP"
fi

# P-03 initialize
echo "[P-03] initialize"
BODY=$(curl -s -X POST "$MCP" \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $AUTH" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"1.0"}}}' \
  --max-time 15 || echo "{}")
PROTO=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('result',{}).get('protocolVersion','MISSING'))" 2>/dev/null || echo "PARSE_ERROR")
if [[ "$PROTO" == "2024-11-05" ]]; then
  pass "initialize OK, protocolVersion=$PROTO"
else
  fail "initialize protocolVersion=$PROTO; body=$BODY"
fi

# P-04 tools/list count == 14
echo "[P-04] tools/list (expect 14)"
BODY=$(curl -s -X POST "$MCP" \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $AUTH" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  --max-time 15 || echo "{}")
COUNT=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('result',{}).get('tools',[])))" 2>/dev/null || echo "0")
if [[ "$COUNT" == "14" ]]; then
  pass "tools/list returned $COUNT tools"
else
  NAMES=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print([t['name'] for t in d.get('result',{}).get('tools',[])])" 2>/dev/null || echo "parse error")
  fail "tools/list returned $COUNT tools (expected 14); names=$NAMES"
fi

# P-05 wipeLogs
echo "[P-05] tools/call wipeLogs"
BODY=$(curl -s -X POST "$MCP" \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $AUTH" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wipeLogs","arguments":{}}}' \
  --max-time 15 || echo "{}")
HAS_CONTENT=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print('ok' if d.get('result',{}).get('content') else 'missing')" 2>/dev/null || echo "error")
if [[ "$HAS_CONTENT" == "ok" ]]; then
  pass "wipeLogs returned content"
else
  fail "wipeLogs bad response: $BODY"
fi

# P-06 runNextJSAudit (composite — returns guidance text, no real browser needed)
echo "[P-06] tools/call runNextJSAudit (composite)"
BODY=$(curl -s -X POST "$MCP" \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $AUTH" \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"runNextJSAudit","arguments":{}}}' \
  --max-time 20 || echo "{}")
HAS_TEXT=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); t=d.get('result',{}).get('content',[]); print('ok' if t and t[0].get('text') else 'missing')" 2>/dev/null || echo "error")
if [[ "$HAS_TEXT" == "ok" ]]; then
  pass "runNextJSAudit returned text guidance"
else
  fail "runNextJSAudit bad response: $BODY"
fi

# P-07 getConsoleLogs (real bridge — empty array when no extension, still valid)
echo "[P-07] tools/call getConsoleLogs"
BODY=$(curl -s -X POST "$MCP" \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $AUTH" \
  -d '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"getConsoleLogs","arguments":{}}}' \
  --max-time 15 || echo "{}")
HAS_CONTENT=$(echo "$BODY" | python3 -c "import sys,json; d=json.load(sys.stdin); print('ok' if 'result' in d else 'missing')" 2>/dev/null || echo "error")
if [[ "$HAS_CONTENT" == "ok" ]]; then
  pass "getConsoleLogs returned result (array may be empty)"
else
  fail "getConsoleLogs bad response: $BODY"
fi

# P-08 GET /mcp → 405
echo "[P-08] GET /mcp → expect 405"
HTTP=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$MCP" \
  -H "Authorization: Basic $AUTH" \
  --max-time 10 || echo "000")
if [[ "$HTTP" == "405" ]]; then
  pass "GET /mcp returned HTTP 405"
else
  fail "Expected HTTP 405, got HTTP $HTTP"
fi

echo ""
echo "======================================="
echo " RESULT: PASS=$PASS_COUNT FAIL=$FAIL_COUNT"
echo "======================================="

if [[ "$FAIL_COUNT" -eq 0 ]]; then
  echo " 🟢 ALL $PASS_COUNT PROBES PASSED — P-10 GREEN"
  echo " SR-02 mission status: 🟡 AMBER → 🟢 GREEN"
  exit 0
else
  echo " 🔴 $FAIL_COUNT PROBE(S) FAILED — P-10 RED"
  echo " SR-02 mission status: 🟡 AMBER (unresolved)"
  exit 1
fi
