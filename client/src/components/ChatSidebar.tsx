import { useContext, useEffect, useState } from "react";
import RecentConversation from "./RecentConversation";
import { useNavigate } from "react-router-dom";
import { WebsocketContext } from "./Chat";

export default function ChatSidebar({
  friends,
  colors,
}: {
  friends: string[];
  colors: string[];
}) {
  const ws = useContext(WebsocketContext);
  const [activeConvo, setActiveConvo] = useState(-1);
  const navigate = useNavigate();

  useEffect(() => {}, [ws]);

  const handleClick = (index: number) => {
    if (index === activeConvo) {
      setActiveConvo(-1);
      navigate("/chat");
    } else {
      setActiveConvo(index);
      navigate(`/chat/messages/${friends[index]}`);
    }
  };

  return (
    <>
      <div className="absolute w-[23vw] h-full flex flex-col justify-center items-center gap-4">
        {friends.map((friend, index) => (
          <RecentConversation
            friend={friend}
            index={index}
            color={colors[index]}
            activeIndex={activeConvo}
            onClick={handleClick}
          />
        ))}
      </div>
    </>
  );
}
