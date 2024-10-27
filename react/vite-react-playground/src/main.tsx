import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import {
  createBrowserRouter,
  RouterProvider,
  Route,
  createRoutesFromElements,
} from "react-router-dom";
import ButtonPage from "./pages/Button/ButtonPage.tsx";
import AppPage from "./pages/App/AppPage.tsx";
import ErrorPage from "./pages/Error/ErrorPage.tsx";
import ScriptPage from "./pages/Script/ScriptPage.tsx";

const router = createBrowserRouter(
  createRoutesFromElements(
    <>
      <Route path="/" element={<AppPage />} errorElement={<ErrorPage />} />
      <Route path="button" element={<ButtonPage />} />
      <Route path="script_sync_example" element={<ScriptPage />} />
    </>
  )
);

async function enableMocking() {
  if (process.env.NODE_ENV !== "development") {
    return;
  }

  const { worker } = await import("./mocks/browser");

  // `worker.start()` returns a Promise that resolves
  // once the Service Worker is up and ready to intercept requests.
  return worker.start();
}

enableMocking().then(() => {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <RouterProvider router={router} />
    </StrictMode>
  );
});
