import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// Outside Tauri (plain `pnpm dev` in a browser), serve sample data instead of Rust commands.
const ready =
  import.meta.env.DEV && !("__TAURI_INTERNALS__" in window) ? import("./devMock") : Promise.resolve();

ready.then(() => {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
});
