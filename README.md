# blockchain-hack-warsaw-2026

# DriftMind 

> Language-first AI trading agent on Drift Protocol. Just say what you want to trade.

## What is DriftMind?

DriftMind lets traders describe their strategies in plain English — the AI agent monitors conditions and executes trades on Drift Protocol automatically, 24/7.

**Example:**
```
"Buy 10 SOL with 3x leverage if price drops below $129, stop-loss at $120"
```
Agent parses the intent → monitors every 30 seconds → executes automatically when condition is met.

## Why DriftMind?

- **No forms** — describe intent in plain English, not dropdowns
- **No code** — no bots to write or maintain  
- **No monitoring** — agent executes while you sleep
- **Multi-condition** — combine price, funding rate, time triggers in one sentence

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust + Drift-RS SDK |
| AI Engine | Rust + Claude API |
| Frontend | Next.js + TypeScript |
| Price feeds | Binance API + Drift Oracle |
| Blockchain | Solana (devnet) |
| RPC | Helius |

## Architecture
```
User types strategy
    ↓
AI Engine (Rust + Claude API) — parses to JSON
    ↓
Strategy Engine — monitors conditions every 30s
    ↓
Drift Protocol SDK — executes trade on-chain
    ↓
Dashboard — live PnL and positions
```

## Business Model

- **Drift Builder Codes** — automatic % of every trade routed through DriftMind, settled on-chain
- **Subscription** — $20-50/month for advanced multi-condition strategies
- **Roadmap** — vault strategies with performance fees

## Roadmap

- **Phase 1** — MVP on Drift Protocol (now)
- **Phase 2** — Multi-protocol Solana (Jupiter, Kamino, Marginfi)
- **Phase 3** — Cross-chain (Hyperliquid, Lighter, Aster)
- **Phase 4** — Ethereum ecosystem (GMX, dYdX, Aave)

## Team

PJATK Rust Academic Club — Warsaw, Poland

| Name | Role |
|------|------|
| Jarosław Koźluk | Team Lead |
| Daniel Olczyk | Rust Backend |
| Norbert Olkowski | AI Engine |
| Zuzanna Kościelniak | Rust Backend |
| Łukasz Wiszniewski | Frontend / UX |
| Kacper Szczęsny | Rust Developer |

## Built at

Blockchain Hack Warsaw 2026 — Superteam Poland
