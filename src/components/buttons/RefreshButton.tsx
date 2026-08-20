import { useQueryContext } from "../../state/QueryContext";
import { updateDatabase } from "../../api/tauri";
import { Realm } from "../../types/game";
import { useState } from "react";

// This button lets us manually update the data. I called it all refresh but it
// should actually be update not refresh.
// Also this button should have some way to see if we've requested data within
// the current hour and maybe not let us refresh if we've already refreshed?
// Also also the updateDatabase command should emit something when it's done so
// the UI refreshes everything except the currently selected realm and league.

export function RefreshButton() {
  const { realmId } = useQueryContext();
  // const [activeBtn, setActiveBtn] = useState("");
  const [isRunning, setIsRunning] = useState(false);

  async function handleClick(realmId: Realm) {
    setIsRunning(true);
    updateDatabase(realmId)
      .then((data) => {
        if (data !== "success") {
          throw new Error("Refresh did not succeed.");
        }
      })
      .catch((err) => {
        console.log(err);
        <button className="button refresh-data">Refresh Failed</button>;
      })
      .finally(() => setIsRunning(false));
  }

  if (!realmId) {
    return (
      <button className="button refresh-data" disabled={true}>
        Select Realm
      </button>
    );
  } else {
    return (
      <button
        className="button refresh-data"
        disabled={isRunning}
        onClick={() => {
          handleClick(realmId);
        }}
      >
        {isRunning ? "Updating..." : "Refresh Data"}
      </button>
    );
  }
}
