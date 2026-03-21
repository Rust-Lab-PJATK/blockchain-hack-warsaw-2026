type PortfolioPanelProps = {
  totalPnl: number;
  fund: number;
  connectionMode: "mock" | "http";
  lastError: string | null;
};

export function PortfolioPanel({
  totalPnl,
  fund,
  connectionMode,
  lastError,
}: PortfolioPanelProps) {
  const dailyPnl = totalPnl * 0.36;

  const stats = [
    {
      label: "Total PnL",
      value: `${totalPnl >= 0 ? "+" : ""}$${totalPnl.toFixed(2)}`,
      tone: totalPnl >= 0 ? "positive" : "negative",
    },
    {
      label: "Daily PnL",
      value: `${dailyPnl >= 0 ? "+" : ""}$${dailyPnl.toFixed(2)}`,
      tone: dailyPnl >= 0 ? "positive" : "negative",
    },
    { label: "Margin", value: "$4,280", tone: "neutral" },
    { label: "Leverage", value: "3.4x", tone: "accent" },
    { label: "Exposure", value: "$12.6k", tone: "neutral" },
    { label: "Win Rate", value: "67%", tone: "positive" },
  ] as const;

  const feeds = [
    { label: "SOL Funding", value: "-0.012%", color: "#f87171" },
    {
      label: "BTC Funding",
      value: `${fund >= 0 ? "+" : ""}${fund.toFixed(3)}%`,
      color: fund > 0.04 ? "#f87171" : "#10b981",
    },
    {
      label: "Gateway",
      value: connectionMode === "http" ? "Backend" : "Mock",
      color: connectionMode === "http" ? "#10b981" : "#fbbf24",
    },
    {
      label: "Agent",
      value: lastError ? "Degraded" : "Running",
      color: lastError ? "#f87171" : "#10b981",
    },
  ];

  return (
    <div className="td-portfolio">
      <div className="td-portfolio-header">
        <div className="td-section-title">
          Portfolio
        </div>
      </div>

      <div className="td-portfolio-body">
        <div className="td-portfolio-hero">
          <div className="td-portfolio-hero-label">Total Equity</div>
          <div className="td-portfolio-hero-value">$18,940.22</div>
          <div className="td-portfolio-hero-meta">
            {lastError ? "Live feed delayed" : "Live feed synchronized"}
          </div>
        </div>

        <div className="td-stats-grid">
          {stats.map((s) => (
            <div key={s.label} className={`td-stat-card td-stat-card-${s.tone}`}>
              <div className="td-stat-label">{s.label}</div>
              <div className="td-stat-value">{s.value}</div>
            </div>
          ))}
        </div>

        <div className="td-feed-list">
          {feeds.map((f) => (
            <div key={f.label} className="td-feed-row">
              <span className="td-feed-label">{f.label}</span>
              <span className="td-feed-value" style={{ color: f.color }}>
                {f.value}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
