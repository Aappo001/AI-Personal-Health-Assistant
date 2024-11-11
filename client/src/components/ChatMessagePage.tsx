import { useParams } from "react-router-dom";
import useMessageStore from "../store/hooks/useMessageStore";
import { useUserMapContext, useUserMapDispatchContext } from "./UserMapContext";
import { useContext, useState } from "react";
import { WebsocketContext } from "./Chat";
import SpeechBubble from "./SpeechBubble";
import useUserStore from "../store/hooks/useUserStore";
import { getUserFromId } from "../utils/utils";

export default function ChatMessagePage() {
  const user = useUserStore();
  const messageStore = useMessageStore();
  const userMap = useUserMapContext();
  const updateUserMap = useUserMapDispatchContext();
  const { handleSendMessage } = useContext(WebsocketContext);
  const [message, setMessage] = useState("");
  let { id } = useParams();
  if (!id) {
    window.location.href = "/chat";
    return;
  }

  return (
    <div className="flex flex-col justify-between items-center w-screen h-screen py-32">
      <h1 className="text-6xl text-offwhite">Conversation {id}</h1>
      <div className=" w-2/5 flex flex-col gap-4">
        {messageStore[parseInt(id)]?.map((message, i) => {
          if (userMap[message.userId] === undefined) {
            console.log("UserMap userId is undefined");

            getUserFromId(message.userId)
              .then((unknownUser) => {
                if (!unknownUser) return;
                updateUserMap({ ...userMap, [message.userId]: unknownUser.username });
              })
              .catch((err) => {
                console.log(`Error getting user: ${err}`);
              });
          }
          return (
            <SpeechBubble
              message={message.content}
              from={userMap[message.userId]}
              isFromUser={message.userId === user.id}
              key={`${message.userId}-${i}`}
            />
          );
        })}
      </div>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          setMessage("");
          handleSendMessage(message, parseInt(id));
        }}
        className="bg-[#363131] w-2/5 focus:outline-none rounded-full text-offwhite flex justify-between"
      >
        <input
          type="text"
          name="query"
          placeholder={`Enter Message`}
          className="px-8 py-5 focus:outline-none bg-transparent placeholder:text-offwhite placeholder:text-lg w-5/6"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
        />
        <button
          type="submit"
          className="px-8 py-5 w-32 rounded-full bg-lilac text-main-black font-bold"
        >
          Submit
        </button>
      </form>
    </div>
  );
}
