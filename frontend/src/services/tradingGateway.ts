import { INITIAL_POSITIONS, MOCK_RESPONSES } from "@/services/tradingData";
import {
  getNextMarketTick,
  updatePositionMarks,
} from "@/services/tradingService";
import { Message, Position } from "@/utils/types";

type GatewayMode = "mock" | "http";

export type InitialDashboardState = {
  solPrice: number;
  chg: number;
  fund: number;
  positions: Position[];
  welcomeMessage: Message;
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
  };
};

export type GatewayRequestOptions = {
  signal?: AbortSignal;
  headers?: HeadersInit;
};

export type TradingApiEndpoints = {
  dashboard: string;
  marketTick: string;
  agentParse: string;
};

export const TRADING_API_ENDPOINTS: TradingApiEndpoints = {
  dashboard: "/dashboard",
  marketTick: "/market/tick",
  agentParse: "/agent/parse",
};

export type MarketTickRequest = {
  previousPrice: number;
  previousFunding: number;
  positions: Position[];
};

export type AgentPromptRequest = {
  prompt: string;
};

export type BackendErrorResponse = {
  message?: string;
  code?: string;
};

export type TradingGateway = {
  mode: GatewayMode;
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
    options?: GatewayRequestOptions,
  ) => Promise<AgentReply>;
};

function getRandomMockResponse() {
  const idx = Math.floor(Math.random() * MOCK_RESPONSES.length);
  return MOCK_RESPONSES[idx];
}

function createMockTradingGateway(): TradingGateway {
  return {
    mode: "mock",
    async getInitialDashboardState() {
      return {
        solPrice: 131.42,
        chg: 1.96,
        fund: 0.031,
        positions: INITIAL_POSITIONS,
        welcomeMessage: {
          role: "agent",
          text: "Ask about markets, positions, and risk. I can help in real time.",
        },
      };
    },
    async getMarketTick(previousPrice, previousFunding, positions) {
      const nextTick = getNextMarketTick(previousPrice, previousFunding);

      return {
        nextPrice: nextTick.nextPrice,
        pctChange: nextTick.pctChange,
        nextFunding: nextTick.nextFunding,
        positions: updatePositionMarks(positions, nextTick.nextPrice),
      };
    },
    async sendPrompt() {
      await new Promise((resolve) => window.setTimeout(resolve, 500));
      return getRandomMockResponse();
    },
  };
}

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
      const maybeError = (await response.json()) as BackendErrorResponse;
      backendErrorMessage = maybeError.message;
    } catch {
      // Ignore JSON parsing failures and fallback to status-based message.
    }

    throw new Error(
      backendErrorMessage ?? `Gateway request failed: ${response.status}`,
    );
  }

  return (await response.json()) as T;
}

function buildGatewayHeaders(extraHeaders?: HeadersInit): HeadersInit {
  // Backend integration point:
  // Add auth/session headers here when backend enables protected endpoints.
  // Example: Authorization: Bearer <token>, X-Trace-Id: <uuid>
  return {
    ...(extraHeaders ?? {}),
  };
}

function createHttpTradingGateway(baseUrl: string): TradingGateway {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl);

  return {
    mode: "http",
    getInitialDashboardState(options) {
      return fetchJson<InitialDashboardState>(
        `${normalizedBaseUrl}${TRADING_API_ENDPOINTS.dashboard}`,
        {
          signal: options?.signal,
          headers: buildGatewayHeaders(options?.headers),
        },
      );
    },
    getMarketTick(previousPrice, previousFunding, positions, options) {
      const requestBody: MarketTickRequest = {
        previousPrice,
        previousFunding,
        positions,
      };

      return fetchJson<MarketTick>(`${normalizedBaseUrl}${TRADING_API_ENDPOINTS.marketTick}`, {
        method: "POST",
        signal: options?.signal,
        headers: buildGatewayHeaders(options?.headers),
        body: JSON.stringify(requestBody),
      });
    },
    sendPrompt(prompt, options) {
      const requestBody: AgentPromptRequest = { prompt };

      // Backend integration point:
      // If agent endpoint starts streaming (SSE/WebSocket), replace this call
      // with a streaming client and push partial tokens into chat state.
      return fetchJson<AgentReply>(`${normalizedBaseUrl}${TRADING_API_ENDPOINTS.agentParse}`, {
        method: "POST",
        signal: options?.signal,
        headers: buildGatewayHeaders(options?.headers),
        body: JSON.stringify(requestBody),
      });
    },
  };
}

export function createTradingGateway(): TradingGateway {
  const apiBaseUrl = process.env.NEXT_PUBLIC_TRADING_API_URL;
  if (apiBaseUrl) {
    return createHttpTradingGateway(apiBaseUrl);
  }
  return createMockTradingGateway();
}
