export type ParsedStrategy = {
  asset: string;
  side: string;
  size: string;
  trigger: string;
  stop_loss: string;
};

export type Message = {
  role: "user" | "agent";
  text: string;
  parsed?: ParsedStrategy;
};

export type Position = {
  market: string;
  side: "long" | "short";
  size: string;
  entry: number;
  mark: number;
  pnl: number;
  status: "triggered" | "manual" | "watching";
};

export type Strategy = {
  asset: string;
  condition: string;
  value: string;
  action: string;
  tags: string[];
  status: "triggered" | "monitoring" | "waiting";
  progress: number;
};
