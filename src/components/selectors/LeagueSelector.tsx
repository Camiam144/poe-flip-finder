import { useQueryContext } from "../../state/QueryContext";
import { useLeagues } from "../../hooks/useLeagues";

// Super similar to RealmSelector, but this gets updated when the Realm changes

export function LeagueSelector() {
  const { realmId, leagueId, setLeagueId } = useQueryContext();
  const { leagues, isLoading, error } = useLeagues(realmId);
  const now = new Date();

  if (error) {
    console.log(error);
    return <span className="selector-error">Couldn't load leagues</span>;
  }

  const disabled = !realmId || isLoading;

  return (
    <select
      className="selector league-selector"
      value={leagueId ?? ""}
      disabled={disabled}
      onChange={(e) => setLeagueId(e.target.value || null)}
    >
      <option value="" disabled>
        {!realmId
          ? "Select a realm first"
          : isLoading
            ? "Loading leagues..."
            : "Select a League"}
      </option>
      {/* I do the filtering here, maybe it should be on the backend? */}
      {leagues
        .filter(
          (league) =>
            (!league.endAt || new Date(league.endAt) >= now) &&
            !(
              league.id.includes("SSF") || league.id.includes("Solo Self-Found")
            ),
        )
        .map((league) => (
          <option key={league.id} value={league.id}>
            {league.name}
          </option>
        ))}
    </select>
  );
}
