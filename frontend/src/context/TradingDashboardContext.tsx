"use client";

import { createContext, ReactNode, useContext } from "react";
import { useTradingDashboard } from "@/hooks/useTradingDashboard";

type TradingDashboardContextValue = ReturnType<typeof useTradingDashboard>;

const TradingDashboardContext = createContext<
  TradingDashboardContextValue | undefined
>(undefined);

export function TradingDashboardProvider({
  children,
}: {
  children: ReactNode;
}) {
  const value = useTradingDashboard();

  return (
    <TradingDashboardContext.Provider value={value}>
      {children}
    </TradingDashboardContext.Provider>
  );
}

export function useTradingDashboardContext() {
  const context = useContext(TradingDashboardContext);

  if (!context) {
    throw new Error(
      "useTradingDashboardContext must be used within TradingDashboardProvider",
    );
  }

  return context;
}
