import { Message, Position } from "@/utils/types";

type GatewayMode = "http";

export type InitialDashboardState = {
  solPrice: number;
  chg: number;
  fund: number;
  positions: Position[];
  initialMessages: Message[];
};

export type MarketTick = {
  nextPrice: number;
  pctChange: number;
  nextFunding: number;
  positions: Position[];
};

export type AgentReply = {
  text: string;
  // Optional backend payload describing an action that must be user-confirmed.
  proposedAction?: {
    label: string;
    summary: string;
    strategyId?: number;
  };
};

export type GatewayRequestOptions = {
  signal?: AbortSignal;
  headers?: HeadersInit;
};

export type TradingApiEndpoints = {
  strategies: string;
  approveStrategy: (id: number) => string;
  chat: string;
  chatHistory: string;
};

export const TRADING_API_ENDPOINTS: TradingApiEndpoints = {
  strategies: "/api/strategies",
  approveStrategy: (id: number) => `/api/strategies/${id}/approve`,
  chat: "/api/chat",
  chatHistory: "/api/chat/history",
};

export type AgentPromptRequest = {
  messages: Array<{
    role: "user" | "assistant";
    content: string;
  }>;
};

export type BackendErrorResponse = {
  message?: string;
  code?: string;
};

export type TradingGateway = {
  mode: GatewayMode;
  getStrategies: (
    options?: GatewayRequestOptions,
  ) => Promise<StrategyDto[]>;
  findLatestPendingStrategyId: (
    options?: GatewayRequestOptions,
  ) => Promise<number | null>;
  getInitialDashboardState: (
    options?: GatewayRequestOptions,
  ) => Promise<InitialDashboardState>;
  getMarketTick: (
    previousPrice: number,
    previousFunding: number,
    positions: Position[],
    options?: GatewayRequestOptions,
  ) => Promise<MarketTick>;
  sendPrompt: (
    prompt: string,
    history: Message[],
    options?: GatewayRequestOptions,
  ) => Promise<AgentReply>;
  approveStrategy: (
    strategyId: number,
    options?: GatewayRequestOptions,
  ) => Promise<void>;
};

type Side = "buy" | "sell";
type StrategyStatus =
  | "waiting"
  | "approved"
  | "triggered"
  | "stopped"
  | "failed"
  | "queued";

type StrategyDto = {
  id: number;
  symbol: string;
  side: string;
  leverage: number;
  price: string | number;
  quantity: string | number;
  status: string;
};

type ChatResponseDto = {
  content: string;
};

type ChatHistoryResponseDto = {
  messages: Array<{
    role: "user" | "assistant";
    content: string;
  }>;
};

function normalizeBaseUrl(url: string) {
  return url.endsWith("/") ? url.slice(0, -1) : url;
}

async function fetchJson<T>(
  input: RequestInfo,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(input, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
  });

  if (!response.ok) {
    let backendErrorMessage: string | undefined;

    try {
      const contentType = response.headers.get("content-type") ?? "";
      if (contentType.includes("application/json")) {
        const maybeError = (await response.json()) as BackendErrorResponse;
        backendErrorMessage = maybeError.message;
      } else {
        const text = await response.text();
        backendErrorMessage = text || undefined;
      }
    } catch {
      // Ignore JSON parsing failures and fallback to status-based message.
    }

    throw new Error(
      backendErrorMessage ?? `Gateway request failed: ${response.status}`,
    );
  }

  return (await response.json()) as T;
}

async function fetchVoid(input: RequestInfo, init?: RequestInit): Promise<void> {
  const response = await fetch(input, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
  });

  if (!response.ok) {
    const text = await response.text().catch(() => "");
    throw new Error(text || `Gateway request failed: ${response.status}`);
  }
}

function buildGatewayHeaders(extraHeaders?: HeadersInit): HeadersInit {
  // Backend integration point:
  // Add auth/session headers here when backend enables protected endpoints.
  // Example: Authorization: Bearer <token>, X-Trace-Id: <uuid>
  return {
    ...(extraHeaders ?? {}),
  };
}

function toNumber(value: string | number | null | undefined, fallback = 0) {
  if (value === null || value === undefined) {
    return fallback;
  }

  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : fallback;
}

function normalizeStrategyStatus(status: string): StrategyStatus {
  const normalized = status.toLowerCase();

  switch (normalized) {
    case "waiting":
    case "approved":
    case "triggered":
    case "stopped":
    case "failed":
    case "queued":
      return normalized;
    default:
      return "waiting";
  }
}

function normalizeSide(side: string): Side {
  return side.toLowerCase() === "buy" ? "buy" : "sell";
}

function toPositionLifecycle(status: string): Position["lifecycle"] {
  const normalizedStatus = normalizeStrategyStatus(status);

  if (
    normalizedStatus === "triggered" ||
    normalizedStatus === "stopped" ||
    normalizedStatus === "failed"
  ) {
    return "processed";
  }

  return "active";
}

function mapStrategyStatusToPositionStatus(status: string): Position["status"] {
  const normalizedStatus = normalizeStrategyStatus(status);

  if (normalizedStatus === "triggered") {
    return "triggered";
  }

  if (normalizedStatus === "approved" || normalizedStatus === "queued") {
    return "manual";
  }

  return "watching";
}

function mapStrategiesToPositions(strategies: StrategyDto[]): Position[] {
  // Exclude pending strategies and split view in UI via lifecycle field.
  return strategies
    .filter(
      (strategy) => normalizeStrategyStatus(strategy.status) !== "waiting",
    )
    .map((strategy) => {
    const entry = toNumber(strategy.price);
    const size = toNumber(strategy.quantity);
    const normalizedSide = normalizeSide(strategy.side);
    const side: Position["side"] = normalizedSide === "buy" ? "long" : "short";
    const leverage = Math.max(strategy.leverage || 1, 1);
    const pnlSign = side === "long" ? 1 : -1;

    return {
      id: strategy.id,
      market: `${strategy.symbol}-PERP`,
      side,
      size: `${size} ${strategy.symbol.replace("USDT", "")}`,
      entry,
      mark: entry,
      pnl: Number((entry * 0.005 * leverage * pnlSign).toFixed(2)),
      status: mapStrategyStatusToPositionStatus(strategy.status),
      lifecycle: toPositionLifecycle(strategy.status),
    };
  });
}

function extractLatestPrice(strategies: StrategyDto[], fallback: number) {
  const firstWithPrice = strategies.find((strategy) => strategy.price !== null);
  if (!firstWithPrice) {
    return fallback;
  }
  return toNumber(firstWithPrice.price, fallback);
}

function mapChatHistoryToMessages(history: ChatHistoryResponseDto): Message[] {
  // UI uses "agent" role while backend uses "assistant".
  return history.messages.map((message) => ({
    role: message.role === "assistant" ? "agent" : "user",
    text: message.content,
  }));
}

function toChatHistory(
  messages: Message[],
): AgentPromptRequest["messages"] {
  // Chat endpoint expects backend role names, so convert UI role names back.
  return messages.map((message) => ({
    role: message.role === "agent" ? "assistant" : "user",
    content: message.text,
  }));
}

function buildCurrentDayRange() {
  // Current backend API requires explicit history window via from/to query params.
  // Keep chat bootstrapping scoped to today's messages only.
  const now = new Date();
  const from = new Date(now);
  from.setHours(0, 0, 0, 0);

  return {
    from: from.toISOString(),
    to: now.toISOString(),
  };
}

function createHttpTradingGateway(baseUrl: string): TradingGateway {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl);

  return {
    mode: "http",
    async getStrategies(options) {
      return fetchJson<StrategyDto[]>(
        `${normalizedBaseUrl}${TRADING_API_ENDPOINTS.strategies}`,
        {
          signal: options?.signal,
          headers: buildGatewayHeaders(options?.headers),
        },
      );
    },
    async findLatestPendingStrategyId(options) {
      const strategies = await fetchJson<StrategyDto[]>(
        `${normalizedBaseUrl}${TRADING_API_ENDPOINTS.strategies}`,
        {
          signal: options?.signal,
          headers: buildGatewayHeaders(options?.headers),
        },
      );

      const pending = strategies.filter(
        (strategy) => normalizeStrategyStatus(strategy.status) === "waiting",
      );

      if (pending.length === 0) {
        return null;
      }

      return pending.reduce((latest, strategy) =>
        strategy.id > latest.id ? strategy : latest,
      ).id;
    },
    async getInitialDashboardState(options) {
      // Bootstrap combines strategy state and recent chat history into one UI snapshot.
      const [strategies, history] = await Promise.all([
        this.getStrategies(options),
        (async () => {
          const { from, to } = buildCurrentDayRange();
          const params = new URLSearchParams({ from, to });

          try {
            return await fetchJson<ChatHistoryResponseDto>(
              `${normalizedBaseUrl}${TRADING_API_ENDPOINTS.chatHistory}?${params.toString()}`,
              {
                signal: options?.signal,
                headers: buildGatewayHeaders(options?.headers),
              },
            );
          } catch {
            return { messages: [] };
          }
        })(),
      ]);

      const positions = mapStrategiesToPositions(strategies);
      const solPrice = extractLatestPrice(strategies, 131.42);
      const initialMessages = mapChatHistoryToMessages(history);
      if (initialMessages.length === 0) {
        initialMessages.push({
          role: "agent",
          text: "Connected to backend. Ask me about strategy status and approvals.",
        });
      }

      return {
        solPrice,
        chg: 0,
        // No dedicated funding endpoint in current contract.
        fund: 0,
        positions,
        initialMessages,
      };
    },
    async getMarketTick(previousPrice, previousFunding, _positions, options) {
      // Contract-only refresh: market view is derived from current strategies list.
      const strategies = await this.getStrategies(options);
      const mappedPositions = mapStrategiesToPositions(strategies);
      const nextPrice = extractLatestPrice(strategies, previousPrice);
      const pctChange =
        previousPrice === 0
          ? 0
          : Number((((nextPrice - previousPrice) / previousPrice) * 100).toFixed(2));

      return {
        nextPrice,
        pctChange,
        nextFunding: previousFunding,
        positions: mappedPositions,
      };
    },
    async sendPrompt(prompt, history, options) {
      // Preserve conversation context by sending full message history required by /api/chat/.
      const requestBody: AgentPromptRequest = {
        messages: [
          ...toChatHistory(history),
          {
            role: "user",
            content: prompt,
          },
        ],
      };

      // Backend integration point:
      // If agent endpoint starts streaming (SSE/WebSocket), replace this call
      // with a streaming client and push partial tokens into chat state.
      const response = await fetchJson<ChatResponseDto>(
        `${normalizedBaseUrl}${TRADING_API_ENDPOINTS.chat}`,
        {
          method: "POST",
          signal: options?.signal,
          headers: buildGatewayHeaders(options?.headers),
          body: JSON.stringify(requestBody),
        },
      );

      return {
        text: response.content,
      };
    },
    async approveStrategy(strategyId, options) {
      // Explicit approval step for pending strategy actions from chat confirmations.
      await fetchVoid(
        `${normalizedBaseUrl}${TRADING_API_ENDPOINTS.approveStrategy(strategyId)}`,
        {
          method: "POST",
          signal: options?.signal,
          headers: buildGatewayHeaders(options?.headers),
        },
      );
    },
  };
}

export function createTradingGateway(): TradingGateway {
  const apiBaseUrl = process.env.NEXT_PUBLIC_TRADING_API_URL ?? "";
  return createHttpTradingGateway(apiBaseUrl);
}
