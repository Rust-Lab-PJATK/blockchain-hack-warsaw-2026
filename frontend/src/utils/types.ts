export type ConfirmationActionStatus =
  | "pending"
  | "confirmed"
  | "cancelled";

export type ConfirmationAction = {
  id: string;
  label: string;
  summary: string;
  status: ConfirmationActionStatus;
};

export type Message = {
  role: "user" | "agent";
  text: string;
  confirmationAction?: ConfirmationAction;
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
