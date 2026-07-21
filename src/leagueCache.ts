import type { RawLeagueApiResponse, Realm } from "./types/game";

const leagueCache = new Map<string, RawLeagueApiResponse>();

export function getCachedLeagues(
  realm: Realm,
): RawLeagueApiResponse | undefined {
  return leagueCache.get(realm);
}

export function setCachedLeagues(
  realm: Realm,
  leagues: RawLeagueApiResponse,
): void {
  leagueCache.set(realm, leagues);
}

// Escape hatch if needed
export function clearLeagueCache(realm?: Realm): void {
  if (realm) {
    leagueCache.delete(realm);
  } else {
    leagueCache.clear();
  }
}
