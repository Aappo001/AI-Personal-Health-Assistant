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
import { ChatHome } from "./components/ChatHome.tsx";
import ChatMessagePage from "./components/ChatMessagePage.tsx";
import ProtectedRoute from "./components/ProtectedRoute.tsx";
import ProfilePage from "./components/ProfilePage.tsx";

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
    children: [
      {
        index: true,
        element: <ChatHome />,
        errorElement: <PageNotFound />,
      },
      {
        path: "/chat/messages/:friend",
        element: <ChatMessagePage />,
        errorElement: <PageNotFound />,
      },
    ],
  },
  {
    path: "/profile/:username",
    element: (
      <ProtectedRoute>
        <ProfilePage />
      </ProtectedRoute>
    ),
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
