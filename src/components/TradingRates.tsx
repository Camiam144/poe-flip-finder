import { useQueryContext } from "../state/QueryContext";
import { useTradingCurrencyRates } from "../hooks/useTradingRates";
import { Realm, TradingCurrencyRates } from "../types/game";

const RatesTable = ({
  tradingRates,
}: {
  tradingRates: TradingCurrencyRates;
}) => (
  <tbody>
    <tr>
      <td>Div</td>
      <td>1 : {tradingRates.div_to_exalt.toFixed(2)}</td>
      <td>Exalt</td>
    </tr>
    <tr>
      <td>Div</td>
      <td>1 : {tradingRates.div_to_chaos.toFixed(2)}</td>
      <td>Chaos</td>
    </tr>
    <tr>
      <td>Chaos:</td>
      <td>1 : {tradingRates.chaos_to_exalt.toFixed(2)}</td>
      <td>Exalt</td>
    </tr>
  </tbody>
);

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
          <td>Choose a Realm and League to display rates.</td>
        </tr>
      </tbody>
    );
  }

  return (
    <>
      {!isLoading ? (
        <RatesTable tradingRates={tradingRates} />
      ) : (
        <tbody>
          <tr>
            <td>Loading...</td>
          </tr>
        </tbody>
      )}
    </>
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
