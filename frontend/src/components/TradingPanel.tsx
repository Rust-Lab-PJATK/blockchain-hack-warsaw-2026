import { Position } from "@/utils/types";
import { Badge } from "@/components/ui/Badge";
import { SideBadge } from "@/components/ui/SideBadge";
import { MiniChart } from "@/components/ui/MiniChart";
import { TIMEFRAME_TABS } from "@/utils/constants";

type TradingPanelProps = {
  solPrice: number;
  chg: number;
  activeTab: string;
  setActiveTab: (tab: string) => void;
  positions: Position[];
};

export function TradingPanel({
  solPrice,
  chg,
  activeTab,
  setActiveTab,
  positions,
}: TradingPanelProps) {
  const isPositive = chg >= 0;

  return (
    <div className="td-trading">
      <div className="td-trading-header">
        <div className="td-price-row">
          <span className="td-market-label">SOL-PERP</span>
          <span className="td-price-value">
            ${solPrice.toFixed(2)}
          </span>
          <span
            className="td-price-change"
            style={{
              color: isPositive ? "var(--td-positive)" : "var(--td-negative)",
              background: isPositive
                ? "rgba(16, 185, 129, 0.1)"
                : "rgba(248, 113, 113, 0.1)",
            }}
          >
            {isPositive ? "+" : ""}
            {chg.toFixed(2)}%
          </span>
        </div>

        <MiniChart price={solPrice} />

        <div className="td-tabs">
          {TIMEFRAME_TABS.map((t) => (
            <div
              key={t}
              onClick={() => setActiveTab(t)}
              className={`td-tab ${activeTab === t ? "td-tab-active" : ""}`}
            >
              {t}
            </div>
          ))}
        </div>
      </div>

      <div className="td-table-wrap">
        <table className="td-table">
          <thead>
            <tr className="td-table-head-row">
              {["Market", "Side", "Size", "Entry", "Mark", "PnL", "Status"].map(
                (h) => (
                  <th key={h} className="td-table-heading">
                    {h}
                  </th>
                ),
              )}
            </tr>
          </thead>
          <tbody>
            {positions.map((p, i) => (
              <tr key={`${p.market}-${i}`} className="td-table-row">
                <td className="td-table-cell td-table-mono">
                  {p.market}
                </td>
                <td className="td-table-cell">
                  <SideBadge side={p.side} />
                </td>
                <td className="td-table-cell td-table-text">
                  {p.size}
                </td>
                <td className="td-table-cell td-table-text td-table-mono">
                  ${p.entry.toFixed(2)}
                </td>
                <td className="td-table-cell td-table-mono">
                  ${p.mark.toFixed(2)}
                </td>
                <td className="td-table-cell">
                  <span
                    className={`td-pnl-value ${
                      p.pnl >= 0 ? "td-pnl-positive" : "td-pnl-negative"
                    }`}
                  >
                    {p.pnl >= 0 ? "+" : ""}${p.pnl.toFixed(2)}
                  </span>
                </td>
                <td className="td-table-cell">
                  <Badge status={p.status} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
