import { LastRefreshedText } from "./LastRefresh";
import { LeagueSelector } from "./selectors/LeagueSelector";
import { RealmSelector } from "./selectors/RealmSelector";

// Tiny little component wrapper to child components

export function TopBar() {
  return (
    <div className="top-bar" style={{ display: "flex" }}>
      <div>
        <RealmSelector />
        <LeagueSelector />
      </div>
      <div>
        <LastRefreshedText />
      </div>
      <div style={{ marginLeft: "auto" }}>RefreshData</div>
    </div>
  );
}
