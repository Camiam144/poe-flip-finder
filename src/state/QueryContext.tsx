import { createContext, useContext, useState, ReactNode } from "react";
import { Realm } from "../types/game";

// This holds query state, stuff that needs to get passed to the backend
// basically anything in this app that needs to go to the backend should live
// in here.

interface QueryState {
  realmId: Realm | null;
  leagueId: string | null;
  setRealmId: (id: Realm | null) => void;
  setLeagueId: (id: string | null) => void;
}

// createContext here is a way to let any descendant component read from this
// object without having to pass down props through every single layer ("prop drilling")

const QueryContext = createContext<QueryState | undefined>(undefined);

export function QueryProvider({ children }: { children: ReactNode }) {
  const [realmId, setRealmIdState] = useState<Realm | null>(null);
  const [leagueId, setLeagueIdState] = useState<string | null>(null);

  // We do things this way because if we swap realms we need to also clear
  // the league state and fetch new leagues
  function setRealmId(id: Realm | null) {
    setRealmIdState(id);
    setLeagueIdState(null);
  }

  const value: QueryState = {
    realmId,
    leagueId,
    setRealmId,
    setLeagueId: setLeagueIdState,
  };

  return (
    <QueryContext.Provider value={value}>{children}</QueryContext.Provider>
  );
}

// Components should call this hook instead of useContext(QueryContext) directly
// so a real error gets thrown instead of silently swallowing it.
export function useQueryContext(): QueryState {
  const ctx = useContext(QueryContext);
  if (!ctx) {
    throw new Error("useQueryContext must be used within a QueryProvider");
  }

  return ctx;
}
