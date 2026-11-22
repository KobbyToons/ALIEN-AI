import Widget from "./components/Widget";
import AgentOverlay from "./components/AgentOverlay";

export default function App() {
  const isOverlay = window.location.search.includes("mode=overlay");

  if (isOverlay) {
    return <AgentOverlay />;
  }

  return (
    <div className="dark select-none">
      <Widget />
    </div>
  );
}

