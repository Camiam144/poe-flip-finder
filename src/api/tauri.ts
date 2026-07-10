// Every tauri command should live in here, that way I only have to register
// them in one place.
import { invoke } from "@tauri-apps/api/core";
import type { RawLeagueApiResponse, Realm } from "../types/game";

export async function fetchLeagues(
  realmId: Realm,
): Promise<RawLeagueApiResponse> {
  return await invoke("get_leagues", { realm: realmId });
}
