import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import App from "./App.tsx";
import PageNotFound from "./components/PageNotFound.tsx";
import Login from "./components/Login.tsx";
import Register from "./components/Register.tsx";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    errorElement: <PageNotFound />, //will load when an error or not found error occurs anywhere in the app
  },
  {
    path: "/login",
    element: <Login />,
    errorElement: <PageNotFound />, //will load when an error or not found error occurs anywhere in the app
  },
  {
    path: "/register",
    element: <Register />,
    errorElement: <PageNotFound />,
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>
);
