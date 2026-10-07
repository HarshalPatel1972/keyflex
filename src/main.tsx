import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "@fontsource-variable/nunito";
import "./styles.css";
import { applyTheme, rememberedTheme } from "./theme";

// Before the first paint, so the window never flashes the wrong colours.
applyTheme(rememberedTheme());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
