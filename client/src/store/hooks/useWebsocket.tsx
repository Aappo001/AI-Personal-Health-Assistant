import { useEffect, useRef, useState } from "react";
import { getJwt, getUserIdFromUsername } from "../../utils/utils";
import {
  wsSendFriendRequest,
  SocketResponse,
  wsRequestConversations,
  wsInviteUsersToConvo,
  wsRequestConversation,
  wsRequestMessages,
} from "../../utils/ws-utils";

const handleMessage = (event: MessageEvent) => {
  console.log("Received websocket response");
  const data = JSON.parse(event.data);
  const type = data.type;
  if (!type) {
    console.log("type field missing from JSON response");
    return;
  }
  switch (type) {
    case SocketResponse.Message:
      console.log(`Received message: ${data.message} from user ${data.userId}`);
      break;
    case SocketResponse.FriendRequest:
      console.log("Friend Request sent or received");
      break;
    case SocketResponse.Generic:
      console.log(`Received Generic Message: ${data.message}`);
      break;
    case SocketResponse.Invite:
      console.log(`Received Invite Message from user id ${data.inviter}`);
      break;
    case SocketResponse.Conversation:
      console.log(`User present in conversation with id ${data.id}`);
      break;
    case SocketResponse.Error:
      console.log(`SocketResponse Error: ${data.message}`);
      break;
    default:
      console.log(`Unknown SocketResponseType: ${type}`);
      console.log(JSON.stringify(data));
  }
};

export default function useWebsocketSetup() {
  const socketRef = useRef<WebSocket | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    console.log("Running chat setup..");
    const url = "ws://localhost:3000/api/ws";
    socketRef.current = new WebSocket(url, [
      "fakeProtocol",
      btoa(`Bearer ${getJwt()}`).replace(/=/g, ""),
    ]);

    socketRef.current.onopen = () => {
      setLoading(false);
      console.log("Websocket connection established?");
    };

    socketRef.current.addEventListener("message", handleMessage);

    return () => {
      console.log("Cleaning up websocket connection");
      socketRef.current?.close();
    };
  }, []);

  return {
    handleSendMessage: (message: string, conversationId: number) => {
      if (!socketRef.current) {
        console.error(`Tried to send message ${message} while WS is null`);
        return;
      }
      socketRef.current.send(
        JSON.stringify({
          type: "SendMessage",
          message: message,
          conversationId: conversationId,
        })
      );
    },

    sendFriendRequest: (username: string) => {
      if (!socketRef.current) {
        console.error(`Error: websocket not initialized`);
        return;
      }

      getUserIdFromUsername(username)
        .then((id) => {
          if (!id || !socketRef.current) return;
          wsSendFriendRequest(socketRef.current, id);
        })
        .catch((err) => console.log(err));
    },

    requestConversations: () => {
      if (!socketRef.current) {
        console.error("Error: Websocket not initialized");
        return;
      }
      wsRequestConversations(socketRef.current);
    },

    inviteUsers: (usernames: string[]) => {
      if (!socketRef.current) {
        console.error("Error: Websocket not initialized");
        return;
      }
      console.log(`InviteUsers input: ${usernames}`);

      // create an array of promises, so that the requests can run concurrently using Promise.all()
      const getIdPromises: Promise<number | undefined>[] = [];
      usernames.forEach((username) => getIdPromises.push(getUserIdFromUsername(username)));

      // fetch all user ids, filter undefined results
      let friendIds: number[] = [];
      Promise.all(getIdPromises)
        .then((ids) => {
          friendIds = ids.filter((id) => id !== undefined);
          //@ts-expect-error it thinks socketRef is null for some reason
          wsInviteUsersToConvo(socketRef.current, friendIds);
        })
        .catch((err) => {
          console.log(`Error getting ids from usernames: ${err}`);
        });
    },

    requestConversation: (id: number) => {
      //@ts-expect-error websocket is not null
      wsRequestConversation(socketRef.current, id);
    },

    requestMessages: (id: number) => {
      //@ts-expect-error websocket is not null
      wsRequestMessages(socketRef.current, id);
    },

    loading,
  };
}
