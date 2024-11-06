import { useEffect, useRef, useState } from "react";
import {
  getJwt,
  getRandomColor,
  getUserFromId,
  getUserIdFromUsername,
} from "../../utils/utils";
import {
  wsSendFriendRequest,
  SocketResponse,
  wsRequestConversations,
  wsInviteUsersToConvo,
  wsRequestConversation,
  wsRequestMessages,
  wsRequestFriends,
} from "../../utils/ws-utils";
import { requestFriendsSchema } from "../../schemas";
import useAppDispatch from "./useAppDispatch";
import { addFriend } from "../friendsSlice";
import { initializeConversationId, pushMessage } from "../messageSlice";
import { AppDispatch } from "../store";

const handleMessage = (type: string, data: any, ws: WebSocket, dispatch: AppDispatch) => {
  console.log("Received websocket response");
  switch (type) {
    case SocketResponse.Message:
      console.log(`Received message: ${data.message} from user ${data.userId}`);

      console.log(JSON.stringify(data));
      dispatch(
        pushMessage({
          id: data.conversationId,
          message: { userId: data.userId, content: data.message },
        })
      );

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
      dispatch(initializeConversationId(data.id));
      wsRequestMessages(ws, data.id);
      break;
    case SocketResponse.Error:
      console.log(`SocketResponse Error: ${data.message}`);
      break;
    case SocketResponse.FriendData:
      console.log(`Friends with user id ${data.id} at ${data.created_at}`);

      const privateUser = requestFriendsSchema.parse(data);
      getUserFromId(privateUser.id)
        .then((user) => {
          if (!user) return;
          const friend = { ...user, color: getRandomColor() };
          dispatch(addFriend(friend));
        })
        .catch((err) => {
          console.error(`Xiao hong shu Error occurred getting user from id:  ${err}`);
          console.log(JSON.stringify(data));
        });
      break;
    default:
      console.log(`Unknown SocketResponseType: ${type}`);
      console.log(JSON.stringify(data));
  }
};

export default function useWebsocketSetup() {
  const socketRef = useRef<WebSocket | null>(null);
  const [loading, setLoading] = useState(true);
  const dispatch = useAppDispatch();

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

    socketRef.current.addEventListener("message", (event: MessageEvent) => {
      const data = JSON.parse(event.data);
      const type = data.type;

      if (!type) {
        console.log("type field missing from JSON response");
        return;
      }

      if (!socketRef.current) return;
      handleMessage(type, data, socketRef.current, dispatch);
    });

    return () => {
      console.log("Cleaning up websocket connection");
      socketRef.current?.close();
    };
  }, []);

  return {
    handleSendMessage: (message: string, conversationId: number, userId: number) => {
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
      dispatch(
        pushMessage({ id: conversationId, message: { userId: userId, content: message } })
      );
    },

    sendFriendRequest: (username: string) => {
      if (!socketRef.current) return;
      getUserIdFromUsername(username)
        .then((id) => {
          if (!id || !socketRef.current) return;
          wsSendFriendRequest(socketRef.current, id);
        })
        .catch((err) => console.log(err));
    },

    requestConversations: () => {
      if (!socketRef.current) return;
      wsRequestConversations(socketRef.current);
    },

    inviteUsers: (usernames: string[]) => {
      if (!socketRef.current) return;
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
      if (!socketRef.current) return;
      wsRequestConversation(socketRef.current, id);
    },

    requestMessages: (id: number) => {
      if (!socketRef.current) return;
      wsRequestMessages(socketRef.current, id);
    },
    requestFriends: () => {
      if (!socketRef.current) return;
      wsRequestFriends(socketRef.current);
    },

    loading,
  };
}
