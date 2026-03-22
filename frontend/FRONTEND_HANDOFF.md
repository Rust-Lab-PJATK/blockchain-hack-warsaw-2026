# Frontend Handoff (dla kolejnego deva)

Ten dokument ma pozwolic wejsc w projekt bez przekopywania calego kodu.

## 1. Stack i uruchomienie

- Framework: Next.js (App Router)
- UI: React + CSS w `src/app/globals.css`
- State/dashboard logic: custom hook + context
- Gateway do backendu: REST przez `fetch`

### Start lokalny

```bash
cd frontend
npm ci
npm run dev
```

App lokalnie: `http://localhost:3000`

## 2. Kluczowe pliki

- `src/app/page.tsx`
  - glowny layout dashboardu
  - wpina `TradingDashboardProvider`
- `src/context/TradingDashboardContext.tsx`
  - provider + hook contextu
- `src/hooks/useTradingDashboard.ts`
  - centralna logika stanu, polling, czat, confirm/cancel
- `src/services/tradingGateway.ts`
  - mapowanie kontraktu backendowego do modelu UI
  - endpointy, fetch, normalizacja enumow
- `src/components/TradingPanel.tsx`
  - tabela trade'ow rozbita na 2 okna: `Active` i `Processed`
- `src/utils/types.ts`
  - typy frontendowe (`Position`, `Message`, itd.)
- `API_CONTRACT.md`
  - kontrakt backendu i endpointy
- `next.config.ts`
  - rewrite `/api/*` -> `http://localhost:5150/api/*` (dev)

## 3. Aktualny flow danych

### Bootstrap

Po wejsciu na strone `useTradingDashboard` robi:

1. `GET /api/strategies/`
2. `GET /api/chat/history?from=...&to=...`

i sklada z tego poczatkowy stan dashboardu.

### Odswiezanie

Co 2 sekundy (`MARKET_TICK_INTERVAL_MS = 2000`) polling:

- `GET /api/strategies/`

### Akcje czatu

- `POST /api/chat/` (pelna historia + nowy prompt)
- `POST /api/strategies/{id}/approve` po `Confirm` (jesli jest `strategyId`)

## 4. Trade'y w UI (wazne)

### Rozbicie listy na 2 okna

W `TradingPanel` pozycje sa rozdzielane po polu `lifecycle`:

- `Active` (lewa kolumna)
- `Processed` (prawa kolumna)

### Skad bierze sie `lifecycle`

W `tradingGateway.ts`, funkcja `toPositionLifecycle(status)`:

- `processed`: `triggered`, `stopped`, `failed`
- `active`: `approved`, `queued`
- `waiting` jest wyciete w filtrze (nie trafia do listy tabel)

### Dodatkowa normalizacja backendu

Backend zwraca enumy czasem jako `Buy`, `Triggered`, `Waiting`.

W `tradingGateway.ts` sa helpery:

- `normalizeStrategyStatus`
- `normalizeSide`

To zabezpiecza refresh i filtrowanie (bez tego lista potrafila znikac).

## 5. Confirm/Approve (wazne dla funkcjonalnosci)

W `useTradingDashboard.ts`:

- potwierdzenie akcji (`confirmAction`) wymaga `strategyId`
- jesli backend nie poda `strategyId`, frontend:
  - blokuje approve
  - pokazuje komunikat `Cannot approve action without strategy ID`

Frontend probuje wyciagnac ID z:

- `response.proposedAction.strategyId`
- tekstu odpowiedzi agenta (`strategy #123`, `id: 123`)
- tekstu promptu

## 6. Konfiguracja API i polaczenia

### Rekomendowane lokalnie

- frontend: `localhost:3000`
- backend: `localhost:5150`
- rewrite w Next przejmuje `/api/*`

### Zmienna env

`NEXT_PUBLIC_TRADING_API_URL` (w `createTradingGateway`) jest obsluzona.

Aktualnie domyslnie frontend korzysta z relatywnego `/api/*` (dziala z rewrite).

## 7. Typowe problemy i szybka diagnostyka

### Problem: po refreshu lista znika

Sprawdz:

1. Czy backend zwraca rekordy:

```bash
curl -sS -L http://localhost:3000/api/strategies/ | jq 'length'
```

2. Czy sa statusy przetwarzalne:

```bash
curl -sS -L http://localhost:3000/api/strategies/ | jq '[.[] | .status]'
```

3. Czy rekordy sa w `Triggered/Approved/Queued/...` (dowolny case jest ok po normalizacji).

### Problem: Confirm nie robi nic

Najczestsza przyczyna: brak `strategyId` w odpowiedzi agenta.

## 8. Co warto zrobic dalej (backlog techniczny)

1. Dodac osobna sekcje "Pending approvals" dla `waiting` zamiast calkowitego ukrycia.
2. Dolozyc endpoint reject po stronie backendu i podpiac go pod `Cancel`.
3. Zastapic polling SSE/WebSocket, gdy backend bedzie gotowy.
4. Uporzadkowac README (obecnie domyslny template Next.js).

## 9. Status na teraz

- UI: stabilny podzial `Active/Processed` dziala.
- Integracja: backend REST podpiety.
- Najwazniejsze edge-case'y (case enumow, brak `strategyId`) sa zabezpieczone.
