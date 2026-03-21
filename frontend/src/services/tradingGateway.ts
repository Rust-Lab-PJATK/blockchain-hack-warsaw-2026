import {
  INITIAL_POSITIONS,
  INITIAL_STRATEGIES,
  MOCK_RESPONSES,
} from "@/services/tradingData";
import {
  getNextMarketTick,
  updatePositionMarks,
} from "@/services/tradingService";
import { Message, ParsedStrategy, Position, Strategy } from "@/utils/types";

type GatewayMode = "mock" | "http";

export type InitialDashboardState = {
  solPrice: number;
  chg: number;
  fund: number;
  positions: Position[];
  strategies: Strategy[];
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
  parsed?: ParsedStrategy;
};

export type TradingGateway = {
  mode: GatewayMode;
  getInitialDashboardState: () => Promise<InitialDashboardState>;
  getMarketTick: (
    previousPrice: number,
    previousFunding: number,
    positions: Position[],
  ) => Promise<MarketTick>;
  sendPrompt: (prompt: string) => Promise<AgentReply>;
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
        strategies: INITIAL_STRATEGIES,
        welcomeMessage: {
          role: "agent",
          text: "Describe your trading strategy and I will parse and monitor it in real time.",
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
    throw new Error(`Gateway request failed: ${response.status}`);
  }

  return (await response.json()) as T;
}

function createHttpTradingGateway(baseUrl: string): TradingGateway {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl);

  return {
    mode: "http",
    getInitialDashboardState() {
      return fetchJson<InitialDashboardState>(`${normalizedBaseUrl}/dashboard`);
    },
    getMarketTick(previousPrice, previousFunding, positions) {
      return fetchJson<MarketTick>(`${normalizedBaseUrl}/market/tick`, {
        method: "POST",
        body: JSON.stringify({
          previousPrice,
          previousFunding,
          positions,
        }),
      });
    },
    sendPrompt(prompt) {
      return fetchJson<AgentReply>(`${normalizedBaseUrl}/agent/parse`, {
        method: "POST",
        body: JSON.stringify({ prompt }),
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
