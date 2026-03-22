# Frontend Production Deployment - DevOps Notes

Ten dokument opisuje gdzie w repo znajduje sie konfiguracja potrzebna do deploymentu frontendu na produkcje oraz jakie sa wymagane ustawienia runtime.

## 1. Gdzie jest konfiguracja deploymentu

### A. Build i start aplikacji

- Plik: `frontend/package.json`
- Sekcja: `scripts`
- Kluczowe komendy:
  - `build` -> `next build`
  - `start` -> `next start`
  - `dev` -> `next dev` (tylko lokalnie)

To jest glowny punkt konfiguracji procesu CI/CD dla frontendu.

### B. Reverse proxy / rewrites do backendu API

- Plik: `frontend/next.config.ts`
- Aktualny rewrite:
  - source: `/api/:path*`
  - destination: `http://localhost:5150/api/:path*`

UWAGA: aktualne ustawienie jest hardcoded na `localhost:5150`, wiec bez zmiany nie nadaje sie na produkcje.

### C. Runtime URL backendu (wykorzystanie env)

- Plik: `frontend/src/services/tradingGateway.ts`
- Linia z konfiguracja:
  - `process.env.NEXT_PUBLIC_TRADING_API_URL ?? ""`

To oznacza:

- Jesli `NEXT_PUBLIC_TRADING_API_URL` jest ustawione, frontend uzyje tej wartosci.
- Jesli nie jest ustawione, frontend wysyla requesty na sciezki relatywne (`/api/...`) na tym samym hostcie co frontend.

### D. Kontrakt API dla integracji

- Plik: `frontend/API_CONTRACT.md`
- Zawiera endpointy i formaty payloadow backendu, potrzebne przy konfiguracji gateway/proxy i testach smoke.

## 2. Wymagane zmienne srodowiskowe (frontend)

Minimalnie:

- `NEXT_PUBLIC_TRADING_API_URL`

Rekomendowane wartosci:

- Wariant 1 (bezposrednio do backendu):
  - `NEXT_PUBLIC_TRADING_API_URL=https://api.twoja-domena.pl`
- Wariant 2 (przez reverse proxy na tym samym hostcie):
  - brak zmiennej lub pusta wartosc i obsluga `/api` przez ingress/proxy

## 3. Co trzeba dopracowac przed produkcja

### A. Usunac hardcoded localhost z `next.config.ts`

Aktualny rewrite na `http://localhost:5150` jest poprawny tylko dla lokalnego developmentu.

Rekomendacja:

- sterowac `destination` przez env (np. `BACKEND_INTERNAL_URL`) albo
- wlaczyc rewrite tylko w dev, a na prod robic routing na poziomie ingress/nginx.

### B. CORS

Dla produkcji trzeba zdecydowac jeden model:

1. Frontend i backend pod tym samym origin przez reverse proxy

- CORS praktycznie znika z perspektywy przegladarki (najbardziej stabilne podejscie)

2. Frontend i backend na roznych domenach

- backend MUSI zwracac poprawne naglowki CORS (`Access-Control-Allow-Origin`, `Methods`, `Headers`) oraz obslugiwac preflight OPTIONS

## 4. Proponowany standard dla DevOps (production)

Najprostszy, stabilny model:

1. Frontend deploy jako Next.js app (`next build` + `next start`)
2. Ingress/reverse proxy routuje:
   - `https://app.twoja-domena.pl/` -> frontend
   - `https://app.twoja-domena.pl/api/*` -> backend service
3. `NEXT_PUBLIC_TRADING_API_URL` zostaje puste (frontend uzywa `/api/...`)
4. Brak cross-origin, brak problemow CORS po stronie przegladarki

## 5. Checklist release dla DevOps

- [ ] `frontend/package.json` - pipeline uzywa `next build` i `next start`
- [ ] `frontend/next.config.ts` - brak hardcoded `localhost` na prod
- [ ] `frontend/src/services/tradingGateway.ts` - poprawnie ustawione `NEXT_PUBLIC_TRADING_API_URL` lub ruch przez `/api`
- [ ] backend dostepny z frontendu (DNS, ingress, TLS)
- [ ] CORS poprawnie ustawiony (jesli rozne originy)
- [ ] smoke test endpointow z `frontend/API_CONTRACT.md`

## 6. Szybkie komendy (lokalna walidacja przed deploy)

```bash
cd frontend
npm ci
npm run build
npm run start
```

Jesli aplikacja startuje i endpointy API odpowiadaja zgodnie z kontraktem, frontend jest gotowy do wdrozenia.
