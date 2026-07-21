import { useEffect, useState } from "react";
import { League, Realm } from "../types/game";
import { fetchLeagues, isFrontendError } from "../api/tauri";

interface UseLeaguesResult {
  leagues: League[];
  isLoading: boolean;
  error: string | null;
}

// Since we make this depend on the realmId, this will rerun whenever that ID changes
// so we get the update "for free" without having to explicitly call it.
export function useLeagues(realmId: Realm | null): UseLeaguesResult {
  const [leagues, setLeagues] = useState<League[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!realmId) {
      setLeagues([]);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    setError(null);

    fetchLeagues(realmId)
      .then((data) => {
        if (!cancelled) {
          setLeagues(data.leagues);
        }
      })
      .catch((err) => {
        console.log(err);
        if (!cancelled && isFrontendError(err)) {
          setError(
            err.kind === "api"
              ? `GGG had an oopsy: [${err.code}] [${err.message}]`
              : `We had an oopsy: ${err.kind} - ${err.message}`,
          );
        } else {
          setError("Unexpected error oh no");
        }
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [realmId]);

  return { leagues, isLoading, error };
}
