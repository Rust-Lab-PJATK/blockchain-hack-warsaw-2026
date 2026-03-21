import { ParsedStrategy } from "@/utils/types";

export function ParsedCard({ data }: { data: ParsedStrategy }) {
  return (
    <div className="td-parsed-card">
      {Object.entries(data).map(([k, v]) => (
        <div key={k} className="td-parsed-row">
          <span className="td-parsed-key">{k}</span>
          <span
            className={`td-parsed-value ${
              k === "stop_loss" ? "td-parsed-value-stop" : "td-parsed-value-normal"
            }`}
          >
            {v}
          </span>
        </div>
      ))}
    </div>
  );
}
