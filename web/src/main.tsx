import { createRoot } from "react-dom/client";
import { App } from "./App";
import { ErrorScreen } from "./ErrorScreen";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <ErrorScreen>
    <App />
  </ErrorScreen>,
);
