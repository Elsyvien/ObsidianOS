import React from "react";
import ReactDOM from "react-dom/client";
import { getRuntimeMode } from "./api";

async function bootstrap() {
  const runtimeMode = getRuntimeMode();
  const RootComponent =
    runtimeMode === "tauri"
      ? (await import("./App")).default
      : (await import("./landing/LandingPage")).LandingPage;

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <RootComponent />
    </React.StrictMode>,
  );
}

void bootstrap();
