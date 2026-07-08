import { useState } from "react";
// import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";


const REALMS = ["poe1", "xbox", "sony", "poe2"]

function App() {
  const [realm, setRealm] = useState(REALMS[0]);
  const [result, setResult] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);


  async function handleGetLeagues() {
    setLoading(true);
    setError(null);

    try {
      const leagues = await invoke("get_leagues", { realm });
      setResult(JSON.stringify(leagues, null, 2));
    } catch (err) {
      // This is if Tauri responds with an error, might want to make these more legible
      setError(String(err));
    } finally {
      setLoading(false)
    }
  }

  return (
    // Bit of vibecoded UI
    <main className="container" style={{ padding: "2rem" }}>
      <h1>League Tester</h1>

      <div style={{ marginBottom: "1rem" }}>
        <label htmlFor="realm-select" style={{ marginRight: "0.5rem" }}>
          Realm
        </label>
        <select
          id="realm-select"
          value={realm}
          onChange={(e) => setRealm(e.target.value)}>
          {REALMS.map((r) => (
            <option key={r} value={r}>
              {r}
            </option>
          ))}
        </select>
      </div>

      <button onClick={handleGetLeagues} disabled={loading}>
        {loading ? "Loading..." : "Get Leagues"}
      </button>

      {error && (
        <p style={{ color: "red", marginTop: "1rem" }}>Error: {error}</p>
      )}

      <pre
        style={{
          marginTop: "1rem",
          padding: "1rem",
          background: "#f5f5f5",
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
        }}
      >
        {result}
      </pre>
    </main>
  );
}

export default App;
