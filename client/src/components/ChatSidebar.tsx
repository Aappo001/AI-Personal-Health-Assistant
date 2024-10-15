import { useState } from "react";
import RecentConversation from "./RecentConversation";
import { useNavigate } from "react-router-dom";

export default function ChatSidebar({
  friends,
  colors,
}: {
  friends: string[];
  colors: string[];
}) {
  const [activeConvo, setActiveConvo] = useState(-1);
  const navigate = useNavigate();

  const handleClick = (index: number) => {
    if (index === activeConvo) {
      setActiveConvo(-1);
      navigate("/chat");
    } else {
      setActiveConvo(index);
      navigate("/chat/messages");
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
