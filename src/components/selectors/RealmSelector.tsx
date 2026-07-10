import { useQueryContext } from "../../state/QueryContext";
import { Realm } from "../../types/game";

// This is a controlled component, which means the value is driven entirely
// by React state not by DOM. This is so other functions can trust this context
// is correct.

const REALMS: Realm[] = ["poe1", "xbox", "sony", "poe2"];

export function RealmSelector() {
  const { realmId, setRealmId } = useQueryContext();

  return (
    <select
      className="selector realm-selector"
      value={realmId ?? ""}
      // disabled={isLoading}
      onChange={(e) => setRealmId((e.target.value as Realm) || null)}
    >
      <option value="" disabled>
        Select a Realm
      </option>
      {REALMS.map((realm) => (
        <option key={realm} value={realm}>
          {realm}
        </option>
      ))}
    </select>
  );
}
