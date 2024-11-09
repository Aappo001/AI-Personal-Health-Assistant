import { Outlet } from "react-router-dom";
import Background from "./Background";
import ChatSidebar from "./ChatSidebar";
import useWebsocketSetup from "../store/hooks/useWebsocket";
import { createContext } from "react";
import useUserStore from "../store/hooks/useUserStore";
import UserMapContext from "./UserMapContext";

type WebsocketContextType = ReturnType<typeof useWebsocketSetup>;
export const WebsocketContext = createContext<WebsocketContextType>(
  {} as WebsocketContextType
);

export default function Chat() {
  const ws = useWebsocketSetup();
  const userStore = useUserStore();

  return (
    <WebsocketContext.Provider value={ws}>
      <UserMapContext initialState={{ [userStore.id]: userStore.username }}>
        <Background color="black">
          <div className="relative h-screen">
            <ChatSidebar />
            <Outlet />
          </div>
        </Background>
      </UserMapContext>
    </WebsocketContext.Provider>
  );
}
