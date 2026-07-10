export type Realm = "poe1" | "xbox" | "sony" | "poe2";

export interface RawLeagueApiResponse {
  leagues: League[];
}

export interface League {
  id: string;
  name?: string;
  realm?: string;
  url?: string;
  startAt?: string;
  endAt?: string;
  description?: string;
}
