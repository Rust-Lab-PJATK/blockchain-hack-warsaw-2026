import { Strategy } from "@/utils/types";
import { Badge } from "@/components/ui/Badge";

type PortfolioPanelProps = {
  totalPnl: number;
  strategies: Strategy[];
  fund: number;
  connectionMode: "mock" | "http";
  lastError: string | null;
};

export function PortfolioPanel({
  totalPnl,
  strategies,
  fund,
  connectionMode,
  lastError,
}: PortfolioPanelProps) {
  return (
    <div className="td-portfolio">
      <div className="td-portfolio-header">
        <div className="td-section-title">
          Portfolio
        </div>
      </div>

      <div className="td-stats-grid">
        {[
          {
            label: "Total PnL",
            value: `${totalPnl >= 0 ? "+" : ""}$${totalPnl.toFixed(2)}`,
            color: totalPnl >= 0 ? "#10b981" : "#f87171",
          },
          { label: "Margin", value: "$4,280", color: "#fff" },
          {
            label: "Strategies",
            value: String(strategies.length),
            color: "#a78bfa",
          },
          { label: "Win Rate", value: "67%", color: "#10b981" },
        ].map((s) => (
          <div key={s.label} className="td-stat-card">
            <div className="td-stat-label">
              {s.label}
            </div>
            <div className="td-stat-value" style={{ color: s.color }}>
              {s.value}
            </div>
          </div>
        ))}
      </div>

      <div className="td-portfolio-subheader">
        <div className="td-section-title">
          Active Strategies
        </div>
      </div>

      <div className="td-strategy-list">
        {strategies.map((s, i) => (
          <div
            key={`${s.asset}-${i}`}
            className={`td-strategy-card ${
              s.status === "triggered" || s.status === "monitoring"
                ? "td-strategy-card-active"
                : "td-strategy-card-idle"
            }`}
          >
            <div className="td-strategy-head">
              <span className="td-strategy-asset">
                {s.asset}
              </span>
              <Badge status={s.status} />
            </div>

            <div className="td-strategy-line">
              {s.condition} <span className="td-strategy-value">{s.value}</span>{" "}
              {"->"} {s.action}
            </div>

            <div className="td-tags">
              {s.tags.map((t, j) => (
                <span key={`${t}-${j}`} className="td-tag">
                  {t}
                </span>
              ))}
            </div>

            <div className="td-progress-track">
              <div
                className="td-progress-fill"
                style={{
                  background:
                    s.status === "triggered" ? "var(--td-positive)" : "var(--td-accent)",
                  width: `${s.progress}%`,
                }}
              />
            </div>
          </div>
        ))}
      </div>

      {[
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
      ].map((f) => (
        <div key={f.label} className="td-feed-row">
          <span className="td-feed-label">{f.label}</span>
          <span className="td-feed-value" style={{ color: f.color }}>
            {f.value}
          </span>
        </div>
      ))}
    </div>
  );
}
