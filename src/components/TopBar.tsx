import { RefreshButton } from "./buttons/RefreshButton";
import { LastRefreshedText } from "./LastRefresh";
import { LeagueSelector } from "./selectors/LeagueSelector";
import { RealmSelector } from "./selectors/RealmSelector";

// Tiny little component wrapper to child components

export function TopBar() {
  return (
    <div className="top-bar">
      <div>
        <RealmSelector />
      </div>
      <div>
        <LeagueSelector />
      </div>
      <div>
        <LastRefreshedText />
      </div>
      <div>
        <RefreshButton />
      </div>
    </div>
  );
}
