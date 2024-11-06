import { useContext, useEffect, useState } from "react";
import RecentConversation from "./RecentConversation";
import { useNavigate } from "react-router-dom";
import { WebsocketContext } from "./Chat";
import useFriendStore from "../store/hooks/useFriendStore";
import { UserIdMap, useUserMapContext, useUserMapDispatchContext } from "./UserMapContext";

export default function ChatSidebar({
  friends,
  colors,
}: {
  friends: string[];
  colors: string[];
}) {
  const navigate = useNavigate();
  const ws = useContext(WebsocketContext);
  const userMap = useUserMapContext();
  const userMapDispatch = useUserMapDispatchContext();
  const friendStore = useFriendStore();
  const [activeConvo, setActiveConvo] = useState(-1);
  const [loading, setLoading] = useState(false);

  const handleClick = (id: number) => {
    if (id === activeConvo) {
      setActiveConvo(-1);
      navigate("/chat", { replace: true });
    } else {
      setActiveConvo(id);
      const friend = userMap[id];
      if (!friend) return;
      navigate(`/chat/messages/${friend}`);
    }
  };

  useEffect(() => {
    if (friendStore.length > 0) {
      setLoading(false);
      return;
    }
    setLoading(true);
    // ws.requestFriends();
    ws.requestConversations();
  }, [ws, friendStore]);

  useEffect(() => {
    if (friendStore.length === 0) return;
    const userMap: UserIdMap = {};
    friendStore.forEach((friend) => {
      userMap[friend.id] = friend.username;
    });
    userMapDispatch(userMap);
  }, friendStore);

  return (
    <>
      <div className="absolute w-[23vw] h-full flex flex-col justify-center items-center gap-4">
        {loading && <p className="text-3xl text-offwhite">You have no friends!</p>}
        {userMap && <p className="text-xl text-offwhite">{JSON.stringify(userMap)}</p>}
        {friendStore &&
          friendStore.map((friend) => (
            <RecentConversation
              friend={friend}
              activeIndex={activeConvo}
              onClick={handleClick}
            />
          ))}
      </div>
    </>
  );
}
