export type ConfirmationActionStatus =
  | "pending"
  | "confirmed"
  | "cancelled";

export type NotificationTone = "success" | "error" | "warning" | "info";

export type DashboardNotification = {
  id: string;
  title: string;
  message?: string;
  tone: NotificationTone;
};

export type ConfirmationAction = {
  id: string;
  label: string;
  summary: string;
  strategyId?: number;
  status: ConfirmationActionStatus;
};

export type Message = {
  role: "user" | "agent";
  text: string;
  confirmationAction?: ConfirmationAction;
};

export type Position = {
  id?: number;
  market: string;
  side: "long" | "short";
  size: string;
  entry: number;
  mark: number;
  pnl: number;
  status: "triggered" | "manual" | "watching";
  lifecycle: "active" | "processed";
};
