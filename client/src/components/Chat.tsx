import { Outlet } from "react-router-dom";
import Background from "./Background";
import ChatSidebar from "./ChatSidebar";

export default function Chat() {
  return (
    <Background color="black">
      <div className="relative h-screen">
        <ChatSidebar />
        <div className="flex flex-col justify-center items-center w-screen h-screen">
          <Outlet />
        </div>
      </div>
    </Background>
  );
}
