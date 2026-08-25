export type Realm = "poe1" | "xbox" | "sony" | "poe2";

export interface RawLeagueApiResponse {
  leagues: League[];
}

export type FrontendError =
  | { kind: "network"; message: string }
  | { kind: "parse"; message: string }
  | { kind: "api"; code: number; message: string }
  | { kind: "database"; message: string }
  | { kind: "invalidInput";  message: string }
  | { kind: "other"; message: string };

export interface League {
  id: string;
  name?: string;
  realm?: string;
  url?: string;
  startAt?: string;
  endAt?: string;
  description?: string;
}

export interface TradingCurrencyRates {
  div_to_exalt: number;
  div_to_chaos: number;
  chaos_to_exalt: number;
}

export type UpdateOutcome = "Success" | "NoUpdateNeeded";
