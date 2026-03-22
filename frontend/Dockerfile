FROM node:24-alpine AS base

WORKDIR /app

ARG PNPM_VERSION=8.15.8
RUN corepack enable && corepack prepare pnpm@${PNPM_VERSION} --activate

FROM base AS deps

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

FROM deps AS builder

ARG NEXT_PUBLIC_TRADING_API_URL
ENV NEXT_PUBLIC_TRADING_API_URL=${NEXT_PUBLIC_TRADING_API_URL}

COPY . .
RUN pnpm build

FROM base AS runner

ENV NODE_ENV=production
ENV HOSTNAME=0.0.0.0
ENV PORT=80
ENV NEXT_PUBLIC_TRADING_API_URL=

COPY --from=deps /app/node_modules ./node_modules
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/next.config.ts ./next.config.ts

EXPOSE 80
CMD ["pnpm", "start"]
