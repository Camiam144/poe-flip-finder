import { useEffect, useState } from "react";
import { FrontendError, League, Realm, TradingCurrencyRates } from "../types/game";
import { fetchTradingCurrencyRates } from "../api/tauri";

interface TradingCurrencyResult {
  tradingRates: TradingCurrencyRates;
  isLoading: boolean;
  error: FrontendError | null;
}

// Since we make this depend on the realmId & leagueID, this will rerun whenever that ID changes
// so we get the update "for free" without having to explicitly call it.
export function useTradingCurrencyRates(
  realmId: Realm | null,
  leagueId: League["id"] | null,
): TradingCurrencyResult {
  const [tradingRates, setRates] = useState<TradingCurrencyRates>({
    exalt_per_div: 0,
    chaos_per_div: 0,
    exalt_per_chaos: 0,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<FrontendError | null>(null);

  useEffect(() => {
    if (!realmId || !leagueId) {
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    setError(null);

    fetchTradingCurrencyRates(realmId, leagueId)
      .then((data) => {
        if (!cancelled) {
          setRates(data);
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

  return { tradingRates, isLoading, error };
}
