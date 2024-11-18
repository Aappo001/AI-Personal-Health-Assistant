import { useParams } from "react-router-dom";
import { useUserMapContext, useUserMapDispatchContext } from "./UserMapContext";
import { useContext, useEffect, useRef, useState } from "react";
import { WebsocketContext } from "./Chat";
import SpeechBubble from "./SpeechBubble";
import useUserStore from "../store/hooks/useUserStore";
import { getUserFromId } from "../utils/utils";
import Toggle from "./Toggle";
import useConversationStore from "../store/hooks/useConversationStore";

export default function ChatMessagePage() {
  const user = useUserStore();
  const conversationStore = useConversationStore();
  const userMap = useUserMapContext();
  const updateUserMap = useUserMapDispatchContext();
  const { handleSendMessage } = useContext(WebsocketContext);
  const [message, setMessage] = useState("");
  const [newTitle, setNewTitle] = useState("");
  const [editEnabled, setEditEnabled] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  //@ts-expect-error setSelectedModel currently not used
  const [selectedModel, setSelectedModel] = useState(3);
  const [aiEnabled, setAiEnabled] = useState(false);
  const { renameConversation } = useContext(WebsocketContext);
  let { id } = useParams();
  if (!id) {
    window.location.href = "/chat";
    return;
  }
  const defaultTitle = `Conversation ${id}`;

  const handleSubmit = (e: React.ChangeEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (aiEnabled) {
      handleSendMessage(message, parseInt(id), selectedModel);
    } else {
      handleSendMessage(message, parseInt(id));
    }
    setMessage("");
  };

  const handleRenameSubmit = () => {
    renameConversation(parseInt(id), newTitle);
    setEditEnabled(false);
  };

  useEffect(() => {
    if (editEnabled && inputRef.current) {
      inputRef.current.focus();
    }
  }, [editEnabled]);

  useEffect(() => {
    setNewTitle(conversationStore[parseInt(id)]?.title ?? defaultTitle);
    setEditEnabled(false);
  }, [id]);

  return (
    <div className="flex flex-col justify-between items-center w-screen h-screen py-32">
      <div className="flex gap-6">
        {editEnabled ? (
          <>
            <input
              type="text"
              value={newTitle}
              onChange={(e) => setNewTitle(e.target.value)}
              className="text-offwhite px-5 py-3 bg-transparent text-6xl focus:outline-none focus:outline-lilac "
              ref={inputRef}
            ></input>
            <div className="flex flex-col justify-between items-center">
              <img
                src="/check.svg"
                alt="Confirm"
                height={30}
                width={30}
                className="cursor-pointer"
                onClick={handleRenameSubmit}
              />
              <img
                src="/x.svg"
                alt="Cancel"
                height={30}
                width={30}
                onClick={() => setEditEnabled(false)}
                className="cursor-pointer"
              />
            </div>
          </>
        ) : (
          <>
            <h1 className="text-6xl text-offwhite ">
              {conversationStore[parseInt(id)]?.title}
            </h1>
            <img
              src="/edit.svg"
              alt="Edit Conversation Title"
              width={25}
              height={25}
              onClick={() => {
                setEditEnabled((prev) => !prev);
                setNewTitle(conversationStore[parseInt(id)]?.title || "");
              }}
            />
          </>
        )}
      </div>
      <div className=" w-2/5 flex flex-col gap-4">
        {conversationStore[parseInt(id)]?.messages?.map((message, i) => {
          if (message.userId !== undefined && userMap[message.userId] === undefined) {
            console.log("UserMap userId is undefined");

            getUserFromId(message.userId)
              .then((unknownUser) => {
                if (!unknownUser || !message.userId) return;
                updateUserMap({ ...userMap, [message.userId]: unknownUser.username });
              })
              .catch((err) => {
                console.log(`Error getting user: ${err}`);
              });
          }

          const from = message.userId ? userMap[message.userId] : "AI";
          return (
            <SpeechBubble
              message={message.content}
              from={from}
              isFromUser={message.userId === user.id}
              key={`${message.userId}-${i}`}
            />
          );
        })}
      </div>
      <form
        onSubmit={handleSubmit}
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
      <div className="flex flex-col justify-center items-center">
        {aiEnabled && <h1 className=" text-green-600 text-xl">AI ENABLED</h1>}
        <Toggle action={setAiEnabled} />
      </div>
    </div>
  );
}
