import Background from "./Background";
import useWebsocketSetup from "../store/hooks/useWebsocket";
import { useState } from "react";
import { useSelector } from "react-redux";
import { Rootstate } from "../store/store";

export default function WebsocketTesting() {
  const {
    handleSendMessage,
    sendFriendRequest,
    requestConversations,
    inviteUsers,
    requestConversation,
    requestMessages,
    requestFriends,
    requestFriendRequests,
    loading,
  } = useWebsocketSetup();

  const [inviteUser, setInviteUser] = useState("");
  const [convoId, setConvoId] = useState(0);
  const [message, setMessage] = useState("");
  const friends = useSelector((state: Rootstate) => state.friendsState.friends);
  const messages = useSelector((state: Rootstate) => state.messageState);

  return (
    <>
      <Background>
        {friends.length > 0 &&
          friends.map((friend) => (
            <h1 className="text-3xl text-offwhite">Friends with {friend.username}</h1>
          ))}
        <div className="w-full flex flex-col justify-center items-center">
          <h1 className=" text-3xl text-offwhite">
            {loading ? "Websocket connection loading...." : "Websocket connection established"}
          </h1>
          <button
            onClick={() => {
              sendFriendRequest(inviteUser, true);
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Send Friend Req to username {inviteUser}
          </button>
          <button
            onClick={() => {
              sendFriendRequest(inviteUser, true);
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Accept Friend Req to username {inviteUser}
          </button>
          <button
            onClick={() => {
              requestConversations();
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Request Conversations
          </button>
          <button
            onClick={() => {
              inviteUsers([inviteUser]);
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Invite {inviteUser ? inviteUser : "???"} to conversation
          </button>
          <input
            type="text"
            placeholder="Enter Invite User"
            onChange={(e) => setInviteUser(e.target.value)}
            className="px-3 py-3 focus:outline-none bg-offwhite text-main-black rounded-sm"
          />
          <button
            onClick={() => requestConversation(convoId)}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Get conversation with id {convoId ? convoId : "???"}
          </button>
          <button
            onClick={() => requestMessages(convoId)}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Request Messages with id {convoId ? convoId : "???"}
          </button>

          <input
            type="text"
            placeholder="Enter Conversation Id"
            onChange={(e) => setConvoId(parseInt(e.target.value))}
            className="px-3 py-3 focus:outline-none bg-offwhite text-main-black rounded-sm mb-5"
          />

          <input
            type="text"
            placeholder="Enter Message"
            onChange={(e) => setMessage(e.target.value)}
            className="px-3 py-3 focus:outline-none bg-offwhite text-main-black rounded-sm"
          />

          <button
            onClick={() => {
              handleSendMessage(message, convoId);
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Send a message to convo id {convoId ? convoId : "???"}
          </button>
          <button
            onClick={() => {
              requestFriends();
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Request Friends
          </button>
          <button
            onClick={() => {
              requestFriendRequests();
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Request Friend Requests
          </button>
          {messages.messages &&
            Object.entries(messages.messages).map(([conversationId, content]) => (
              <h1 className="text-xl text-offwhite">
                ID: {conversationId}, Message: {JSON.stringify(content)}
              </h1>
            ))}
        </div>
      </Background>
    </>
  );
}
