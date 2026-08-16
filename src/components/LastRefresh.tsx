import { useQueryContext } from "../state/QueryContext";
import { useLastRefresh } from "../hooks/useLastUpdated";

export function LastRefreshedText() {
  const { realmId } = useQueryContext();
  const { lastRefresh, isLoading, error } = useLastRefresh(realmId);

  if (error) {
    console.log(error);
    return <span className="refresh-error">Couldn't get refresh</span>;
  }

  return <>{!isLoading ? lastRefresh : "Loading..."}</>;
}
