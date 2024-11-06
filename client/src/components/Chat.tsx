import { Outlet } from "react-router-dom";
import Background from "./Background";
import ChatSidebar from "./ChatSidebar";
import { generateRandomColorArray } from "../utils/utils";
import useWebsocketSetup from "../store/hooks/useWebsocket";
import { createContext } from "react";
import { useSelector } from "react-redux";
import { Rootstate } from "../store/store";
import useUserStore from "../store/hooks/useUserStore";

type WebsocketContextType = ReturnType<typeof useWebsocketSetup>;
export const WebsocketContext = createContext<WebsocketContextType>(
  {} as WebsocketContextType
);

type UserIdMap = {
  [id: number]: string
}

export const UserIdMapContext = createContext<UserIdMap>({} as UserIdMap)

export default function Chat() {
  const ws = useWebsocketSetup();
  const userStore = useUserStore() 
  const friends = ["Levi", "Marco", "Karin", "Olivia", "O'saa", "Daan"];
  const colors = generateRandomColorArray(friends.length);

  return (
    <WebsocketContext.Provider value={ws}>
      <UserIdMapContext.Provider value={{[userStore.id]: userStore.username}}>
      <Background color="black">
        <div className="relative h-screen">
          <ChatSidebar friends={friends} colors={colors} />
          <Outlet />
        </div>
      </Background>
      </UserIdMapContext.Provider>
    </WebsocketContext.Provider>
  );
}
