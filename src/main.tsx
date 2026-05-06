import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";
import "./styles/keyboard.css";
import "./styles/components.css";
import "./styles/overview.css";
import "./styles/heatmap.css";
import "./styles/gaps.css";
import "./styles/per-app.css";
import "./styles/history.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
