import { useContext, useEffect, useState } from "react";
import RecentConversation from "./RecentConversation";
import { useNavigate } from "react-router-dom";
import { WebsocketContext } from "./Chat";
import useFriendStore from "../store/hooks/useFriendStore";
import { UserIdMap, useUserMapContext, useUserMapDispatchContext } from "./UserMapContext";
import useMessageStore from "../store/hooks/useMessageStore";

export default function ChatSidebar() {
  const navigate = useNavigate();
  const ws = useContext(WebsocketContext);
  const userMap = useUserMapContext();
  const userMapDispatch = useUserMapDispatchContext();
  const friendStore = useFriendStore();
  const messageStore = useMessageStore();
  const [activeConvo, setActiveConvo] = useState(-1);
  const [loading, setLoading] = useState(false);

  const handleClick = (id: number) => {
    if (id === activeConvo) {
      setActiveConvo(-1);
      navigate("/chat", { replace: true });
    } else if (id === -2) {
      setActiveConvo(id);
      navigate("/chat/friends");
    } else {
      setActiveConvo(id);
      navigate(`/chat/messages/${id}`);
    }
  };

  useEffect(() => {
    if (friendStore.length > 0) {
      setLoading(false);
      return;
    }
    setLoading(true);
    ws.requestFriends();
    ws.requestConversations();
  }, [ws, friendStore]);

  useEffect(() => {
    if (friendStore.length === 0) return;
    const newUserMap: UserIdMap = {};
    friendStore.forEach((friend) => {
      userMap[friend.id] = friend.username;
    });
    userMapDispatch({ ...userMap, ...newUserMap });
  }, [friendStore]);

  return (
    <>
      <div className="absolute w-[23vw] h-full flex flex-col justify-center items-center gap-4">
        {loading && <p className="text-3xl text-offwhite">You have no friends!</p>}
        {userMap && <p className="text-xl text-offwhite">{JSON.stringify(userMap)}</p>}
        <div
          className={`w-10/12 bg-main-grey rounded-lg cursor-pointer ${
            activeConvo === -2 && "bg-slate-700"
          }`}
          onClick={() => handleClick(-2)}
        >
          <p className="text-xl text-offwhite p-4">Friends List</p>
        </div>
        {Object.entries(messageStore).map(([conversationId, messages]) => (
          <RecentConversation
            id={parseInt(conversationId)}
            activeIndex={activeConvo}
            onClick={handleClick}
            key={`convo-${conversationId}`}
            recentMessage={
              messages
                ? messages[messages.length - 1].content.slice(0, 16)
                : "No previous messages"
            }
          />
        ))}
      </div>
    </>
  );
}
