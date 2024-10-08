import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import PageNotFound from "./components/PageNotFound.tsx";
import App from "./App.tsx";
import { createBrowserRouter, RouterProvider } from "react-router-dom";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    errorElement: <PageNotFound />, //will load when an error or not found error occurs anywhere in the app
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>
);
