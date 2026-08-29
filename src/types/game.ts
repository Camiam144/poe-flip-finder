export type FrontendError =
  | { kind: "network"; message: string }
  | { kind: "parse"; message: string }
  | { kind: "api"; code: number; message: string }
  | { kind: "database"; message: string }
  | { kind: "invalidInput";  message: string }
  | { kind: "other"; message: string };

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

export type TradingCurrencyType =
  | { type: "Other"; name: string }
  | { type: "Exalt"}
  | { type: "Chaos"}
  | { type: "Divine"}

export interface TradingCurrencyRates {
  div_to_exalt: number;
  div_to_chaos: number;
  chaos_to_exalt: number;
}

export type UpdateOutcome = "Success" | "NoUpdateNeeded";

// this could be improved for sure.
export interface ArbitrageOpportunity {
  path: TradingCurrencyType[],
  high_ratios: number[],
  low_ratios: number[],
  volumes: number[],
}
