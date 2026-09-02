import { useEffect, useState } from "react";
import { FrontendError, League, Realm, OpportunityDisplay} from "../types/game";
import { fetchArbitrage } from "../api/tauri";

interface ArbitrageResult {
  opportunities: OpportunityDisplay[];
  isLoading: boolean;
  error: FrontendError | null;
}

// Since we make this depend on the realmId & leagueID, this will rerun whenever that ID changes
// so we get the update "for free" without having to explicitly call it.
// TODO: Eventually I will need to add in a timestamp or some other better way to process this.
export function useArbitrageOpportunity(
  realmId: Realm | null,
  leagueId: League["id"] | null,
): ArbitrageResult {
  const [opportunities, setOpps] = useState<OpportunityDisplay[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<FrontendError | null>(null);

  useEffect(() => {
    if (!realmId || !leagueId) {
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    setError(null);

    fetchArbitrage(realmId, leagueId)
      .then((data) => {
        if (!cancelled) {
          setOpps(data);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          console.log(err);
          setError(err);
        }
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [realmId, leagueId]);

  return { opportunities, isLoading, error };
}
