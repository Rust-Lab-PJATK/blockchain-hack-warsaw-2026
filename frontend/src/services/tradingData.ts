import { ParsedStrategy, Position, Strategy } from "@/utils/types";

export const MOCK_RESPONSES: Array<{ text: string; parsed: ParsedStrategy }> = [
  {
    text: "Strategy parsed. Monitoring SOL every 30s.",
    parsed: {
      asset: "SOL-PERP",
      side: "LONG 3x",
      size: "10 SOL",
      trigger: "price < $129",
      stop_loss: "$120.00",
    },
  },
  {
    text: "Strategy parsed. I'll short BTC when funding crosses your threshold.",
    parsed: {
      asset: "BTC-PERP",
      side: "SHORT 2x",
      size: "0.1 BTC",
      trigger: "funding > 0.05%",
      stop_loss: "$72,000",
    },
  },
  {
    text: "Got it. Strategy added and active.",
    parsed: {
      asset: "ETH-PERP",
      side: "LONG 4x",
      size: "1 ETH",
      trigger: "price < $3,200",
      stop_loss: "$3,100",
    },
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

export const INITIAL_STRATEGIES: Strategy[] = [
  {
    asset: "SOL-PERP",
    condition: "price <",
    value: "$129",
    action: "LONG 3x",
    tags: ["SL $120", "10 SOL"],
    status: "triggered",
    progress: 100,
  },
  {
    asset: "BTC-PERP",
    condition: "funding >",
    value: "0.05%",
    action: "SHORT 2x",
    tags: ["Now: 0.031%"],
    status: "monitoring",
    progress: 62,
  },
  {
    asset: "ETH-PERP",
    condition: "price <",
    value: "$3,200",
    action: "LONG 4x",
    tags: ["SL $3,100", "1 ETH"],
    status: "waiting",
    progress: 20,
  },
];
