import { useContext, useEffect, useState } from "react";
import RecentConversation from "./RecentConversation";
import { useNavigate } from "react-router-dom";
import { UserIdMapContext, WebsocketContext } from "./Chat";
import useFriendStore from "../store/hooks/useFriendStore";

export default function ChatSidebar({
  friends,
  colors,
}: {
  friends: string[];
  colors: string[];
}) {
  const navigate = useNavigate();
  const ws = useContext(WebsocketContext);
  const userIdMap = useContext(UserIdMapContext);
  const friendStore = useFriendStore();
  const [activeConvo, setActiveConvo] = useState(-1);
  const [loading, setLoading] = useState(false);

  const handleClick = (index: number) => {
    if (index === activeConvo) {
      setActiveConvo(-1);
      navigate("/chat", { replace: true });
    } else {
      setActiveConvo(index);
      navigate(`/chat/messages/${friends[index]}`);
    }
  };

  useEffect(() => {
    if (friendStore.length > 0) {
      setLoading(false);
      return;
    }
    setLoading(true);
    ws.requestFriends();
  }, [ws, friendStore]);

  return (
    <>
      <div className="absolute w-[23vw] h-full flex flex-col justify-center items-center gap-4">
        {loading && <p className="text-3xl text-offwhite">You have no friends!</p>}
        {friendStore &&
          friendStore.map((friend, index) => (
            <RecentConversation
              friend={friend.username}
              index={index}
              color={friend.color}
              activeIndex={activeConvo}
              onClick={handleClick}
            />
          ))}
      </div>
    </>
  );
}
