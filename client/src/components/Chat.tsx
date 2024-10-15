import { Outlet } from "react-router-dom";
import Background from "./Background";
import ChatSidebar from "./ChatSidebar";
import { generateRandomColorArray } from "../utils/utils";

export default function Chat() {
  const friends = ["Levi", "Marco", "Karin", "Olivia", "O'saa", "Daan"];
  const colors = generateRandomColorArray(friends.length);

  return (
    <Background color="black">
      <div className="relative h-screen">
        <ChatSidebar friends={friends} colors={colors} />
        <Outlet />
      </div>
    </Background>
  );
}
