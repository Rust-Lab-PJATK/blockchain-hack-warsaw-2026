import { NAV_ITEMS } from "@/utils/constants";

type TopbarProps = {
  activeNav: string;
  setActiveNav: (nav: string) => void;
};

export function Topbar({ activeNav, setActiveNav }: TopbarProps) {
  return (
    <div className="td-topbar">
      <div className="td-brand">
        DRIFT<span className="td-brand-accent">MIND</span>
      </div>

      <div className="td-nav">
        {NAV_ITEMS.map((n) => (
          <div
            key={n}
            onClick={() => setActiveNav(n)}
            className={`td-nav-item ${activeNav === n ? "td-nav-item-active" : ""}`}
          >
            {n}
          </div>
        ))}
      </div>

      <button type="button" className="td-wallet-btn">
        <div className="td-wallet-dot" />
        8xK2...4mPQ
      </button>
    </div>
  );
}
