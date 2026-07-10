import { LeagueSelector } from "./selectors/LeagueSelector";
import { RealmSelector } from "./selectors/RealmSelector";

// Tiny little component wrapper to child components

export function TopBar() {
  return (
    <div className="top-bar">
      <RealmSelector />
      <LeagueSelector />
    </div>
  );
}
