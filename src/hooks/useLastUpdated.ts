import { useEffect, useState } from "react";
import { fetchLastRefresh } from "../api/tauri";
import { Realm } from "../types/game";

interface LastRefreshResult {
  lastRefresh: string;
  isLoading: boolean;
  error: string | null;
}

// We make this depend on realm ID so we get a "free" firing to the db I guess?
export function useLastRefresh(realmId: Realm | null): LastRefreshResult {
  const [lastRefresh, setLastRefresh] = useState("never");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!realmId) {
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    setError(null);
    fetchLastRefresh(realmId)
      .then((data) => {
        if (!cancelled) {
          setLastRefresh(data);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          console.log(err);
          setError(err);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [realmId]);
  return { lastRefresh, isLoading, error };
}
