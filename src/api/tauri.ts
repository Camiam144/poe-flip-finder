// Every tauri command should live in here, that way I only have to register
// them in one place.
import { invoke } from "@tauri-apps/api/core";
import { getCachedLeagues, setCachedLeagues } from "../leagueCache";
import type {
  FrontendError,
  RawLeagueApiResponse,
  Realm,
  TradingCurrencyRates,
} from "../types/game";

export function isFrontendError(err: unknown): err is FrontendError {
  return typeof err === "object" && err !== null && "kind" in err;
}

export async function fetchLeagues(
  realmId: Realm,
): Promise<RawLeagueApiResponse> {
  const cached = getCachedLeagues(realmId);
  if (cached) {
    return cached;
  }
  const leagueResponse = await invoke<RawLeagueApiResponse>("get_leagues", {
    realm: realmId,
  });

  setCachedLeagues(realmId, leagueResponse);
  return leagueResponse;
}

export async function fetchTradingCurrencyRates(
  realmId: Realm,
  leagueId: string,
): Promise<TradingCurrencyRates> {
  const tradingRates = await invoke<TradingCurrencyRates>("get_rates", {
    realm: realmId,
    league: leagueId,
  });
  return tradingRates;
}

export async function fetchLastRefresh(realmId: Realm): Promise<string> {
  const lastRefreshTime = await invoke<string>("get_most_recent_update_time", {
    realm: realmId,
  });
  return lastRefreshTime;
}
