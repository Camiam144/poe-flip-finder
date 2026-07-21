export type Realm = "poe1" | "xbox" | "sony" | "poe2";

export interface RawLeagueApiResponse {
  leagues: League[];
}

export type FrontendError =
  | { kind: "network"; message: string }
  | { kind: "parse"; message: string }
  | { kind: "api"; code: number; message: string };

export interface League {
  id: string;
  name?: string;
  realm?: string;
  url?: string;
  startAt?: string;
  endAt?: string;
  description?: string;
}
