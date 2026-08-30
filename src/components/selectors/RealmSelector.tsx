import { useState } from "react";
import { useQueryContext } from "../../state/QueryContext";
import { Realm } from "../../types/game";

// This is a controlled component, which means the value is driven entirely
// by React state not by DOM. This is so other functions can trust this context
// is correct.

// const REALMS: Realm[] = ["poe1", "xbox", "sony", "poe2"];
// const REALMS: Realm[] = ["poe1", "poe2"];

export function RealmSelector() {
  const { setRealmId } = useQueryContext();
  const [activeBtn, setActiveBtn] = useState("");

  return (
    <div className="btn realm-selector">
      <button
        className="button realm-selector"
        value={"poe1"}
        onClick={(e) => {
          setActiveBtn("btnPoe1");
          setRealmId(e.currentTarget.value as Realm);
        }}
        style={{
          fontWeight: activeBtn === "btnPoe1" ? "bold" : "normal",
          opacity: activeBtn === "btnPoe1" ? 1 : 0.5,
          transition: "opacity 0.3s ease",
        }}
      >
        POE 1
      </button>
      <button
        className="button realm-selector"
        value={"poe2"}
        onClick={(e) => {
          setActiveBtn("btnPoe2");
          setRealmId(e.currentTarget.value as Realm);
        }}
        style={{
          fontWeight: activeBtn === "btnPoe2" ? "bold" : "normal",
          opacity: activeBtn === "btnPoe2" ? 1 : 0.5,
          transition: "opacity 0.3s ease",
        }}
      >
        POE 2
      </button>
    </div>
  );
}
