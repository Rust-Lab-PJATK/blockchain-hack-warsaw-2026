export type Message = {
  role: "user" | "agent";
  text: string;
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
