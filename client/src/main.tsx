import { StrictMode } from "react";
import { Provider } from "react-redux";
import { store } from "./store/store.ts";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import App from "./App.tsx";
import PageNotFound from "./components/PageNotFound.tsx";
import Login from "./components/Login.tsx";
import Register from "./components/Register.tsx";
import Chat from "./components/Chat.tsx";

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
  {
    path: "/chat",
    element: <Chat />,
    errorElement: <PageNotFound />,
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider store={store}>
      <RouterProvider router={router} />
    </Provider>
  </StrictMode>
);
