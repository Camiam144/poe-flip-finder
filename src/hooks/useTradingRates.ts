import { useEffect, useState } from "react";
import { League, Realm, TradingCurrencyRates } from "../types/game";
import { fetchTradingCurrencyRates } from "../api/tauri";

interface TradingCurrencyResult {
  tradingRates: TradingCurrencyRates;
  isLoading: boolean;
  error: string | null;
}

// Since we make this depend on the realmId & leagueID, this will rerun whenever that ID changes
// so we get the update "for free" without having to explicitly call it.
export function useTradingCurrencyRates(
  realmId: Realm | null,
  leagueId: League["id"] | null,
): TradingCurrencyResult {
  const [tradingRates, setRates] = useState<TradingCurrencyRates>({
    div_to_exalt: 0,
    div_to_chaos: 0,
    chaos_to_exalt: 0,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
          setError(String(err));
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
