import { Position } from "@/utils/types";

export const MOCK_RESPONSES: Array<{ text: string }> = [
  {
    text: "Monitoring SOL momentum and funding. Current trend still constructive.",
  },
  {
    text: "BTC funding is elevated. Consider reducing exposure if volatility expands.",
  },
  {
    text: "Risk check complete. Keep a tighter stop while liquidity stays thin.",
  },
];

export const INITIAL_POSITIONS: Position[] = [
  {
    market: "SOL-PERP",
    side: "long",
    size: "10 SOL",
    entry: 128.9,
    mark: 131.42,
    pnl: 75.6,
    status: "triggered",
  },
  {
    market: "ETH-PERP",
    side: "short",
    size: "2 ETH",
    entry: 3420,
    mark: 3398,
    pnl: 44,
    status: "manual",
  },
  {
    market: "BTC-PERP",
    side: "long",
    size: "0.05 BTC",
    entry: 67200,
    mark: 66850,
    pnl: -87.5,
    status: "watching",
  },
];

