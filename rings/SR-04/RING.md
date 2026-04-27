# RING — SR-04 (Tunnel)

| Field | Value |
|-------|-------|
| Metal | 🥈 Silver |
| Level | 04 — Funnel |
| Depends on | SR-00 |

## Types exported
- `TAILSCALE_BINARY` — path to Tailscale CLI
- `start_funnel(port: u16)` — Start funnel on port
- `stop_funnel()` — Stop funnel
- `get_status()` — Get current status

## Invariants
- R1: no imports from other rings
- L6: Pure Rust only
