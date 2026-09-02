import { useArbitrageOpportunity } from "../hooks/useArbitrage";
import { useQueryContext } from "../state/QueryContext";
import { OpportunityDisplay, Realm, TradingCurrencyType } from "../types/game";


function getTradingCurrencyName(tct: TradingCurrencyType): string  {
   return (tct.type == "Other" ? tct.name : tct.type.toString())
}

function formatRatio(ratio: number): string {
  let outstr: string;

  if (ratio === 1) {
    outstr = "1 : 1";
  } else if (ratio > 1) {
    outstr = `${(ratio).toFixed(2)} : 1`;
  } else if (ratio < 1 && ratio > 0) {
    outstr = `1 : ${(1/ratio).toFixed(2)}`;
  } else {
    console.log(`Invalid ratio: ${ratio}`);
    outstr = "err : err";
  }

  // if (ratio) {
  //   outstr = `${ratio}`
  // } else {
  //   outstr = "err"
  // }

  return outstr
}

function OppRow({ opp }: {opp: OpportunityDisplay}) {

  return (
    <tr>
      <td>
        {getTradingCurrencyName(opp.path[0])}
      </td>
      <td>
        {formatRatio(opp.high_ratios[0])}
      </td>
      <td>
        {getTradingCurrencyName(opp.path[1])}
      </td>
      <td>
        {formatRatio(opp.high_ratios[1])}
      </td>
      <td>
        {getTradingCurrencyName(opp.path[2])}
      </td>
      <td>
        {(opp.roi).toLocaleString('en-US', {style: 'percent'})}
      </td>
      <td>
        {opp.profit.toLocaleString('en-US', {minimumFractionDigits: 2, maximumFractionDigits: 2})}
      </td>
    </tr>
  )
}

function ArbitrageTable(
  {
    realmId,
    leagueId,
  }: {
    realmId: Realm | null;
    leagueId: string | null;
    }) {
  const rows = [];
  let { opportunities, error } = useArbitrageOpportunity(realmId, leagueId);

  if (!realmId || !leagueId) {
    rows.push(
      <tr key={"loading"}>
        <td>
        {"Pick a league"}
        </td>
      </tr>
    )
  }

  else if (error) {
    rows.push(
      <tr key={"error"}>
        <td>{error.message}</td>
      </tr>
    );
  }

  else {
    for (const opp of opportunities){
      if (opp.path.length < 3) {
        continue
      }
      const key = opp.path.map((e) => e.type == "Other" ? e.name : e.type.toString()).join("|");
      rows.push(
        <OppRow opp={opp} key={key} />
      );
    }
  }



  return (
      <tbody>
        {rows}
      </tbody>
  )
}
export function SortableArbitrageTable() {
  let { realmId, leagueId } = useQueryContext();

  return (
    <div className="table-container arbitrage-table-container">
      <table className="table arbitrage-table">
        <caption>Arbitrage Opportunities</caption>
        <thead>
          <tr>
            <th>Start</th>
            <th>Rate</th>
            <th>Mid</th>
            <th>Rate</th>
            <th>End</th>
            <th>ROI %</th>
            <th>Profit?</th>
          </tr>
        </thead>
        <ArbitrageTable realmId={realmId} leagueId={leagueId} />
      </table>
    </div>
  );
}
