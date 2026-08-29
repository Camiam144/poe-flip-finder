import { useArbitrageOpportunity } from "../hooks/useArbitrage";
import { useQueryContext } from "../state/QueryContext";
import { ArbitrageOpportunity, Realm } from "../types/game";


function OppRow({ opp }: {opp: ArbitrageOpportunity}) {

  return (
    <tr>
      <td>
        {opp.path[0].type}
      </td>
      <td>
        {opp.high_ratios[0]}
      </td>
      <td>
        {opp.path[1].name}
      </td>
      <td>
        {opp.high_ratios[1]}
      </td>
      <td>
        {opp.path[2].type}
      </td>
      <td>
        {opp.high_ratios[2]}
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
  let { opportunities, isLoading, error } = useArbitrageOpportunity(realmId, leagueId);

  if (!realmId || !leagueId) {
    rows.push(
      <tr>
        "Pick a league"
      </tr>
    )
  }

  else if (error) {
    rows.push(
      <tr>
        {error.message}
      </tr>
    )
  }

  else {
    for (const opp of opportunities){
      if (opp.path.length < 3) {
        continue
      }
      const key = opp.path.map((e) => e.type == "Other" ? e.name : e.type.toString()).join("|");
      rows.push(
        <OppRow opp={opp} key={key} />
      )
    }
  }



  return (
    <table>
      <thead>
        <tr>
          <th>Start</th>
          <th>Rate</th>
          <th>Mid</th>
          <th>Rate</th>
          <th>End</th>
        </tr>
      </thead>
      <tbody>
        {rows}
      </tbody>
    </table>
  )
}
export function SortableArbitrageTable() {
  let { realmId, leagueId } = useQueryContext();

  return (
    <table className="table arbitrage-table">
      <caption>Arbitrage Opportunities</caption>
      <ArbitrageTable realmId={realmId} leagueId={leagueId} />
    </table>
  );
}
