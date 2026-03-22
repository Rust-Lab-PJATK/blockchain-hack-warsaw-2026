"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createTradingGateway } from "@/services/tradingGateway";
import { DashboardNotification, Message, NotificationTone, Position } from "@/utils/types";

const MARKET_TICK_INTERVAL_MS = 2000;
const NOTIFICATION_TIMEOUT_MS = 4500;
const MAX_NOTIFICATIONS = 4;

function inferConfirmableAction(prompt: string) {
  const normalizedPrompt = prompt.trim();
  if (!normalizedPrompt) {
    return null;
  }

  // Safety gate: these commands imply account-affecting changes and should
  // only run after explicit user confirmation.
  const requiresConfirmationPattern =
    /(buy|sell|open|close|long|short|stop|take profit|tp|sl|leverage|size)/i;

  if (!requiresConfirmationPattern.test(normalizedPrompt)) {
    return null;
  }

  const strategyMatch = normalizedPrompt.match(/\bstrategy\s*#?\s*(\d+)\b/i);
  // If prompt references a strategy number, bind confirmation to backend approve endpoint.
  const strategyId = strategyMatch ? Number(strategyMatch[1]) : undefined;

  return {
    label: "Proposed trading action",
    summary: normalizedPrompt,
    strategyId,
  };
}

function extractStrategyIdFromText(text: string) {
  const directMatch = text.match(/\bstrategy\s*#?\s*(\d+)\b/i);
  if (directMatch) {
    return Number(directMatch[1]);
  }

  const idMatch = text.match(/\b(?:id|strategy_id)\s*[:#]?\s*(\d+)\b/i);
  if (idMatch) {
    return Number(idMatch[1]);
  }

  return undefined;
}

async function resolveLatestPendingStrategyIdFromApi() {
  const response = await fetch("/api/strategies", {
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Strategies request failed: ${response.status}`);
  }

  const payload = (await response.json()) as Array<{
    id: number;
    status: string;
  }>;

  const waiting = payload.filter(
    (strategy) => strategy.status.toLowerCase() === "waiting",
  );

  if (waiting.length === 0) {
    return null;
  }

  return waiting.reduce((latest, strategy) =>
    strategy.id > latest.id ? strategy : latest,
  ).id;
}

function createActionId() {
  return `action-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function createNotificationId() {
  return `notif-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function getPendingConfirmationSummary(messages?: Message[]) {
  if (!Array.isArray(messages)) {
    return null;
  }

  const pendingAction = messages.find(
    (message) => message.confirmationAction?.status === "pending",
  )?.confirmationAction;

  return pendingAction?.summary ?? null;
}

export function useTradingDashboard() {
  const gatewayRef = useRef(createTradingGateway());
  const isTickInFlightRef = useRef(false);
  const priceRef = useRef(131.42);
  const fundRef = useRef(0.031);
  const positionsRef = useRef<Position[]>([]);

  const [activeNav, setActiveNav] = useState("Trade");
  const [activeTab, setActiveTab] = useState("15m");
  const [solPrice, setSolPrice] = useState(131.42);
  const [chg, setChg] = useState(1.96);
  const [fund, setFund] = useState(0.031);
  const [positions, setPositions] = useState<Position[]>([]);
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [notifications, setNotifications] = useState<DashboardNotification[]>([]);
  const [isBootstrapping, setIsBootstrapping] = useState(true);
  const [isSending, setIsSending] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);

  const msgsRef = useRef<HTMLDivElement | null>(null);
  const notificationTimersRef = useRef<Map<string, number>>(new Map());

  const dismissNotification = useCallback((notificationId: string) => {
    setNotifications((prev) =>
      prev.filter((notification) => notification.id !== notificationId),
    );

    const timer = notificationTimersRef.current.get(notificationId);
    if (timer) {
      window.clearTimeout(timer);
      notificationTimersRef.current.delete(notificationId);
    }
  }, []);

  const pushNotification = useCallback(
    (title: string, tone: NotificationTone, message?: string) => {
      const notificationId = createNotificationId();

      setNotifications((prev) => {
        const next = [{ id: notificationId, title, tone, message }, ...prev];
        return next.slice(0, MAX_NOTIFICATIONS);
      });

      const timeoutId = window.setTimeout(() => {
        dismissNotification(notificationId);
      }, NOTIFICATION_TIMEOUT_MS);

      notificationTimersRef.current.set(notificationId, timeoutId);
    },
    [dismissNotification],
  );

  useEffect(() => {
    priceRef.current = solPrice;
  }, [solPrice]);

  useEffect(() => {
    fundRef.current = fund;
  }, [fund]);

  useEffect(() => {
    positionsRef.current = positions;
  }, [positions]);

  useEffect(() => {
    const timers = notificationTimersRef.current;

    return () => {
      for (const timeoutId of timers.values()) {
        window.clearTimeout(timeoutId);
      }
      timers.clear();
    };
  }, []);

  useEffect(() => {
    let active = true;
    const bootstrapAbortController = new AbortController();

    async function bootstrap() {
      try {
        const data = await gatewayRef.current.getInitialDashboardState({
          signal: bootstrapAbortController.signal,
        });

        if (!active) {
          return;
        }

        setSolPrice(data.solPrice);
        setChg(data.chg);
        setFund(data.fund);
        setPositions(data.positions);
        setMessages(Array.isArray(data.initialMessages) ? data.initialMessages : []);
        setLastError(null);
      } catch {
        if (!active) {
          return;
        }

        setLastError("Unable to load dashboard data.");
        pushNotification(
          "Dashboard connection failed",
          "error",
          "Could not load initial data from backend.",
        );
      } finally {
        if (active) {
          setIsBootstrapping(false);
        }
      }
    }

    bootstrap();

    return () => {
      active = false;
      bootstrapAbortController.abort();
    };
  }, [pushNotification]);

  useEffect(() => {
    if (isBootstrapping) {
      return;
    }

    // Backend integration point:
    // Current implementation polls every MARKET_TICK_INTERVAL_MS.
    // When backend exposes push transport (SSE/WebSocket), replace this block
    // with subscription setup and dispatch incoming ticks directly into state.
    const id = window.setInterval(async () => {
      if (isTickInFlightRef.current) {
        return;
      }

      isTickInFlightRef.current = true;

      try {
        const nextTick = await gatewayRef.current.getMarketTick(
          priceRef.current,
          fundRef.current,
          positionsRef.current,
        );

        setSolPrice(nextTick.nextPrice);
        setChg(nextTick.pctChange);
        setFund(nextTick.nextFunding);
        setPositions(nextTick.positions);
        setLastError(null);
      } catch {
        setLastError("Market feed temporarily unavailable.");
      } finally {
        isTickInFlightRef.current = false;
      }
    }, MARKET_TICK_INTERVAL_MS);

    return () => window.clearInterval(id);
  }, [isBootstrapping]);

  useEffect(() => {
    const node = msgsRef.current;
    if (!node) {
      return;
    }
    node.scrollTop = node.scrollHeight;
  }, [messages]);

  const pendingActionSummary = useMemo(
    () => getPendingConfirmationSummary(messages),
    [messages],
  );
  const hasPendingConfirmation = pendingActionSummary !== null;

  const send = useCallback(async () => {
    const prompt = input.trim();
    if (!prompt || isSending) {
      return;
    }

    if (hasPendingConfirmation) {
      // Safety lock: while there is a pending trade action, block new commands
      // until user explicitly confirms or cancels that action.
      setLastError("Confirm or cancel the pending action before sending a new command.");
      pushNotification(
        "Action pending",
        "warning",
        "Confirm or cancel the current action before sending a new command.",
      );
      return;
    }

    setMessages((prev) => [...prev, { role: "user", text: prompt }]);
    setInput("");

    setIsSending(true);

    try {
      // Backend integration point:
      // For streamed responses, append partial chunks to the last agent message
      // and finalize once server sends completion event.
      // Current REST contract accepts full chat history in every request.
      const response = await gatewayRef.current.sendPrompt(prompt, messages);

      // Confirmation integration point:
      // Backend can return response.proposedAction. Until then, infer candidate
      // actions from user prompt and gate them behind explicit confirmation.
      const proposedAction =
        response.proposedAction ?? inferConfirmableAction(prompt);
      const inferredStrategyId =
        proposedAction?.strategyId ??
        extractStrategyIdFromText(response.text) ??
        extractStrategyIdFromText(prompt);

      const agentMessage: Message = {
        role: "agent",
        text: proposedAction
          ? `${response.text}\n\nThis action requires your confirmation before execution.`
          : response.text,
        confirmationAction: proposedAction
          ? {
              id: createActionId(),
              label: proposedAction.label,
              summary: proposedAction.summary,
              strategyId: inferredStrategyId,
              status: "pending",
            }
          : undefined,
      };

      setMessages((prev) => [...prev, agentMessage]);
      setLastError(null);
    } catch {
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          text: "I could not reach the agent service. Please retry.",
        },
      ]);
      setLastError("Message service unavailable.");
      pushNotification(
        "Message send failed",
        "error",
        "Agent service is unavailable. Please try again.",
      );
    } finally {
      setIsSending(false);
    }
  }, [hasPendingConfirmation, input, isSending, messages, pushNotification]);

  const confirmAction = useCallback((actionId: string) => {
    // Execution gate:
    // This callback is the only place where a pending chatbot action should be
    // promoted to executable. Hook backend execution call here.
    void (async () => {
      const action = messages.find(
        (message) => message.confirmationAction?.id === actionId,
      )?.confirmationAction;

      if (!action) {
        return;
      }

      let strategyId: number | null =
        typeof action.strategyId === "number" ? action.strategyId : null;

      if (strategyId === null) {
        strategyId = await resolveLatestPendingStrategyIdFromApi();
      }

      if (typeof strategyId !== "number") {
        setLastError("No pending strategy found to approve.");
        setMessages((prev) => [
          ...prev,
          {
            role: "agent",
            text:
              "I could not find any pending strategy to approve. Create a strategy first, then confirm.",
          },
        ]);
        pushNotification(
          "Approval blocked",
          "warning",
          "No pending strategy found.",
        );
        return;
      }

      try {
        await gatewayRef.current.approveStrategy(strategyId);

        setMessages((prev) => {
          const nextMessages = prev.map((message) => {
            if (message.confirmationAction?.id !== actionId) {
              return message;
            }

            return {
              ...message,
              confirmationAction: {
                ...message.confirmationAction,
                status: "confirmed" as const,
              },
            };
          });

          return [
            ...nextMessages,
            {
              role: "agent",
              text: `Confirmed. Strategy #${strategyId} approved.`,
            },
          ];
        });

        setLastError(null);
        pushNotification(
          "Action confirmed",
          "success",
          `Strategy #${strategyId} approved.`,
        );
      } catch {
        setLastError("Unable to approve strategy on backend.");
        setMessages((prev) => [
          ...prev,
          {
            role: "agent",
            text: "Approval failed. Please try again.",
          },
        ]);
        pushNotification(
          "Approval failed",
          "error",
          "Backend rejected or could not process the approval.",
        );
      }
    })();
  }, [messages, pushNotification]);

  const cancelAction = useCallback((actionId: string) => {
    setMessages((prev) => {
      const nextMessages = prev.map((message) => {
        if (message.confirmationAction?.id !== actionId) {
          return message;
        }

        return {
          ...message,
          confirmationAction: {
            ...message.confirmationAction,
            status: "cancelled" as const,
          },
        };
      });

      return [
        ...nextMessages,
        {
          role: "agent",
          text: "Action cancelled. No trade was executed.",
        },
      ];
    });
    pushNotification("Action cancelled", "info", "No trade has been executed.");
  }, [pushNotification]);

  const totalPnl = useMemo(
    () => positions.reduce((sum, position) => sum + position.pnl, 0),
    [positions],
  );

  return {
    activeNav,
    setActiveNav,
    activeTab,
    setActiveTab,
    solPrice,
    chg,
    fund,
    positions,
    totalPnl,
    input,
    setInput,
    messages,
    send,
    confirmAction,
    cancelAction,
    msgsRef,
    isBootstrapping,
    isSending,
    lastError,
    hasPendingConfirmation,
    pendingActionSummary,
    connectionMode: gatewayRef.current.mode,
    notifications,
    dismissNotification,
  };
}
