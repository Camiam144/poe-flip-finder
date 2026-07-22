import { TopBar } from "./components/TopBar";
import { QueryProvider } from "./state/QueryContext";
import "./App.css";
import { TradingRateTable } from "./components/TradingRates";

function App() {
  return (
    // Allegedly QueryProvider holds everything that needs the realm or league
    // and anything can go inside here and have access to the realm/league
    <QueryProvider>
      <div className="app">
        <TopBar />
        <main className="app-main">
          {/* <p className="placeholder">Pick a realm and league</p> */}
          <TradingRateTable />
        </main>
      </div>
    </QueryProvider>
  );
}

export default App;
