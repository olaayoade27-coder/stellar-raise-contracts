import React from "react";
import ReactDOM from "react-dom/client";
import { FrontendGlobalErrorBoundary } from "./components/frontend_global_error";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <FrontendGlobalErrorBoundary>
      <App />
    </FrontendGlobalErrorBoundary>
  </React.StrictMode>,
);
