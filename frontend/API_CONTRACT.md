# Backend API Contract (Frontend Integration)

Zakres dokumentu opiera sie na aktualnych endpointach HTTP z kontrolerow.

## Base URL

- Local: `http://localhost:5150`

## Endpoint Summary

- `POST /api/strategies/`
- `GET /api/strategies/`
- `POST /api/strategies/{id}/approve`
- `GET /api/strategies/condition-variables`
- `POST /api/chat/`
- `GET /api/chat/history?from=<RFC3339>&to=<RFC3339>`

Dodatkowo backend wystawia:

- `POST /mcp` (RMCP Streamable HTTP service, nie jest to standardowy JSON REST endpoint)

## Shared Types

### Side

```json
"buy" | "sell"
```

### OrderType

```json
"limit" | "market" | "stop_limit"
```

### StrategyStatus

```json
"waiting" | "approved" | "triggered" | "stopped" | "failed" | "queued"
```

### ChatMessage

```json
{
  "role": "string",
  "content": "string"
}
```

### Strategy (response model)

```json
{
  "id": 123,
  "symbol": "SOLUSDT",
  "side": "buy",
  "order_type": "limit",
  "leverage": 5,
  "price": "125.50",
  "quantity": "2.00",
  "status": "waiting",
  "condition": "price < 120",
  "stop_loss_pct": "5.0",
  "stop_loss_price": "119.0",
  "scheduled_at": "2026-03-22T12:00:00+01:00",
  "executed_at": null,
  "queued_at": null
}
```

Uwaga: pola typu decimal (`price`, `quantity`, `stop_loss_pct`, `stop_loss_price`) frontend powinien obslugiwac jako `string | number` i mapowac jawnie.

## 1) Create Strategy

- Method: `POST`
- Path: `/api/strategies/`
- Body JSON:

```json
{
  "symbol": "SOLUSDT",
  "side": "buy",
  "order_type": "limit",
  "leverage": 5,
  "price": "125.50",
  "quantity": "2.00",
  "condition": "price < 120",
  "stop_loss_pct": "5.0",
  "stop_loss_price": "119.0",
  "scheduled_at": "2026-03-22T12:00:00+01:00"
}
```

- Required fields:
  - `symbol`
  - `side`
  - `order_type`
  - `leverage`
  - `price`
  - `quantity`
- Optional fields:
  - `condition` (domyslnie pusty string)
  - `stop_loss_pct`
  - `stop_loss_price`
  - `scheduled_at`
- Response `200`:
  - `Strategy`
- Response `400`:
  - plain error string (bad request)
- Response `500`:
  - internal server error

## 2) List Strategies

- Method: `GET`
- Path: `/api/strategies/`
- Response `200`:

```json
[
  {
    "id": 123,
    "symbol": "SOLUSDT",
    "side": "buy",
    "order_type": "limit",
    "leverage": 5,
    "price": "125.50",
    "quantity": "2.00",
    "status": "waiting",
    "condition": "price < 120",
    "stop_loss_pct": "5.0",
    "stop_loss_price": "119.0",
    "scheduled_at": "2026-03-22T12:00:00+01:00",
    "executed_at": null,
    "queued_at": null
  }
]
```

## 3) Approve Strategy

- Method: `POST`
- Path: `/api/strategies/{id}/approve`
- Path params:
  - `id` (int64)
- Body: brak
- Response `200`:
  - `Strategy`
- Response `400`:
  - plain error string

## 4) Condition Variables

- Method: `GET`
- Path: `/api/strategies/condition-variables`
- Response `200`:

```json
[
  {
    "name": "price",
    "type": "number",
    "description": "Current market price of the asset"
  },
  {
    "name": "volume",
    "type": "number",
    "description": "Current trading volume (24h)"
  }
]
```

Aktualnie backend wystawia pelny zestaw:

- `price`
- `volume`
- `high_24h`
- `low_24h`
- `open_24h`
- `change_pct`

## 5) Chat

- Method: `POST`
- Path: `/api/chat/`
- Body JSON:

```json
{
  "messages": [
    { "role": "user", "content": "Create SOL strategy" }
  ]
}
```

- Response `200`:

```json
{
  "content": "assistant response text"
}
```

## 6) Chat History

- Method: `GET`
- Path: `/api/chat/history`
- Query params:
  - `from` (RFC3339 datetime with offset)
  - `to` (RFC3339 datetime with offset)
- Example:
  - `/api/chat/history?from=2026-03-20T00:00:00+01:00&to=2026-03-22T23:59:59+01:00`
- Response `200`:

```json
{
  "messages": [
    { "role": "user", "content": "hello" },
    { "role": "assistant", "content": "hi" }
  ]
}
```

## Suggested TypeScript Interfaces

```ts
export type Side = "buy" | "sell";
export type OrderType = "limit" | "market" | "stop_limit";
export type StrategyStatus = "waiting" | "approved" | "triggered" | "stopped" | "failed" | "queued";

export interface ChatMessage {
  role: string;
  content: string;
}

export interface CreateStrategyPayload {
  symbol: string;
  side: Side;
  order_type: OrderType;
  leverage: number;
  price: string | number;
  quantity: string | number;
  condition?: string;
  stop_loss_pct?: string | number | null;
  stop_loss_price?: string | number | null;
  scheduled_at?: string | null;
}

export interface StrategyDto {
  id: number;
  symbol: string;
  side: Side;
  order_type: OrderType;
  leverage: number;
  price: string | number;
  quantity: string | number;
  status: StrategyStatus;
  condition: string;
  stop_loss_pct: string | number | null;
  stop_loss_price: string | number | null;
  scheduled_at: string | null;
  executed_at: string | null;
  queued_at: string | null;
}

export interface ChatRequestDto {
  messages: ChatMessage[];
}

export interface ChatResponseDto {
  content: string;
}

export interface ChatHistoryResponseDto {
  messages: ChatMessage[];
}
```

## Source of Truth in Code

- Strategies controller: `backend/src/controllers/strategies.rs`
- Chat controller: `backend/src/controllers/chat.rs`
- Chat DTOs: `backend/src/services/llm.rs`
- Strategy entity fields: `backend/src/models/_entities/strategy.rs`
- Enums: `backend/src/models/_entities/sea_orm_active_enums.rs`
- Global routes (`/mcp`, mounted controllers): `backend/src/app.rs`
