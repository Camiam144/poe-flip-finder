import { useQueryContext } from "../state/QueryContext";
import { useTradingCurrencyRates } from "../hooks/useTradingRates";
import { Realm } from "../types/game";

function TradingRates({
  realmId,
  leagueId,
}: {
  realmId: Realm | null;
  leagueId: string | null;
}) {
  let { tradingRates, isLoading, error } = useTradingCurrencyRates(
    realmId,
    leagueId,
  );

  if (!realmId || !leagueId) {
    return (
      <tbody>
        <tr>
          <td>Choose a Realm and League first.</td>
        </tr>
      </tbody>
    );
  }

  return (
    <tbody>
      <tr>
        <td>Div</td>
        <td>1 : {tradingRates.div_to_exalt}</td>
        <td>Exalt</td>
      </tr>
      <tr>
        <td>Div</td>
        <td>1 : {tradingRates.div_to_chaos}</td>
        <td>Chaos</td>
      </tr>
      <tr>
        <td>Chaos:</td>
        <td>1 : {tradingRates.chaos_to_exalt}</td>
        <td>Exalt</td>
      </tr>
    </tbody>
  );
}

export function TradingRateTable() {
  let { realmId, leagueId } = useQueryContext();

  return (
    <table className="table rates-table">
      <caption>Trading Currency Rates</caption>
      <TradingRates realmId={realmId} leagueId={leagueId} />
    </table>
  );
}
