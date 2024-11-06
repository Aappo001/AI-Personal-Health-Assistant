import { useParams } from "react-router-dom";
import useMessageStore from "../store/hooks/useMessageStore";
import { useUserMapContext } from "./UserMapContext";
import { useContext, useState } from "react";
import { WebsocketContext } from "./Chat";

export default function ChatMessagePage() {
  const userMap = useUserMapContext();
  const messageStore = useMessageStore();
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
      {messageStore[parseInt(id)]?.map((message, i) => (
        <p className="text-xl text-offwhite" key={`msg-${i}`}>
          From {userMap[message.userId]}: {message.content}
        </p>
      ))}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          setMessage("");
          handleSendMessage(message, parseInt(id));
        }}
        className="bg-[#363131] w-1/2 focus:outline-none rounded-full text-offwhite flex justify-between"
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
