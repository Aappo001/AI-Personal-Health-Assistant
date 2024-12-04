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
import UserHealthForm from "./components/UserForm.tsx";
import FriendsPage from "./components/FriendsPage.tsx";
import WebsocketTesting from "./components/WebsocketTesting.tsx";
import Chart from "./components/rechart.tsx";
import DownloadForm from "./components/Downloadform.tsx";

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
    element: (
      <ProtectedRoute>
        <Chat />
      </ProtectedRoute>
    ),
    errorElement: <PageNotFound />,
    children: [
      {
        index: true,
        element: <ChatHome />,
        errorElement: <PageNotFound />,
      },
      {
        path: "/chat/messages/:id",
        element: <ChatMessagePage />,
        errorElement: <PageNotFound />,
      },
      {
        path: "/chat/friends",
        element: <FriendsPage />,
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
  {
    path: "/ws",
    element: <WebsocketTesting />,
  },
  {
    path: "/healthform",
    element: <UserHealthForm />,
    errorElement: <PageNotFound />,
  },
  {
    path: "/downloadform",
    element: <DownloadForm />,
    errorElement: <PageNotFound />,
  },
  {
  path: "/testChart",
  element: <Chart />, 
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
