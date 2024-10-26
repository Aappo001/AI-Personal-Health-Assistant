import Background from "./Background";
import useWebsocketSetup from "../store/hooks/useWebsocket";
import { useState } from "react";

export default function WebsocketTesting() {
  const { handleSendMessage, sendFriendRequest, requestConversations, inviteUsers, loading } =
    useWebsocketSetup();

  const [inviteUser, setInviteUser] = useState("");

  return (
    <>
      <Background>
        <div className="w-full flex flex-col justify-center items-center">
          <h1 className=" text-3xl text-offwhite">
            {loading ? "Websocket connection loading...." : "Websocket connection established"}
          </h1>
          <button
            onClick={() => {
              handleSendMessage("hello vro");
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Send a message
          </button>
          <button
            onClick={() => {
              sendFriendRequest(inviteUser);
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Send Friend Req to username {inviteUser}
          </button>
          <button
            onClick={() => {
              sendFriendRequest(inviteUser);
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
            className="px-3 py-5 focus:outline-none bg-offwhite text-main-black rounded-sm"
          />
        </div>
      </Background>
    </>
  );
}
