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
  wsRequestFriendRequests,
  wsSendMessage,
  wsLeaveConversation,
} from "../../utils/ws-utils";
import { requestFriendsSchema } from "../../schemas";
import useAppDispatch from "./useAppDispatch";
import { addFriend, removeFriend, upgradeFriendStatus } from "../friendsSlice";
import {
  deleteConversation,
  initializeConversationId,
  pushMessage,
  pushStreamMessage,
} from "../messageSlice";
import { Rootstate } from "../store";
import { Friend } from "../../types";
import { useSelector } from "react-redux";

export default function useWebsocketSetup() {
  const socketRef = useRef<WebSocket | null>(null);
  const [loading, setLoading] = useState(true);
  const dispatch = useAppDispatch();
  const userId = useSelector((state: Rootstate) => state.user.id);

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

      console.log("Received websocket response");
      switch (type) {
        case SocketResponse.Message:
          console.log(`Received message from user ${data.userId}`);

          dispatch(
            pushMessage({
              id: data.conversationId,
              message: { userId: data.userId, content: data.message, fromAi: !data.userId },
            })
          );
          break;

        case SocketResponse.StreamData:
          dispatch(pushStreamMessage({ id: data.conversationId, message: data.message }));
          break;

        case SocketResponse.FriendRequest:
          console.log("SocketResponse: FriendRequest");
          const userIsSender = data.sender_id === userId;
          let id = userIsSender ? data.receiver_id : data.sender_id;

          if (data.status === "Accepted") {
            console.log("Friend Request accepted");
            dispatch(upgradeFriendStatus(id));
            return;
          } else if (data.status === "Rejected") {
            console.log("Friend request rejected");
            dispatch(removeFriend(id));
            return;
          }
          console.log(`Friend Request status: ${data.status}`);

          getUserFromId(id).then((user) => {
            if (!user) return;
            const friend: Friend = {
              ...user,
              status: data.status,
              //looks weird but trust the process
              isSender: !userIsSender,
              color: getRandomColor(),
            };
            dispatch(addFriend(friend));
          });
          break;

        case SocketResponse.Generic:
          console.log(`Received Generic Message: ${data.message}`);
          break;

        case SocketResponse.Invite:
          console.log(`Received Invite Message from user id ${data.inviter}`);
          console.log(`Conversation Invite Data: ${JSON.stringify(data)}`);

          dispatch(initializeConversationId(data.conversation_id));
          break;

        case SocketResponse.Conversation:
          console.log(`User present in conversation with id ${data.id}`);
          dispatch(initializeConversationId(data.id));
          wsRequestMessages(socketRef.current, data.id);
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
              const friend: Friend = { ...user, status: "Accepted", color: getRandomColor() };
              dispatch(addFriend(friend));
            })
            .catch((err) => {
              console.error(`Xiao hong shu Error occurred getting user from id:  ${err}`);
              console.log(JSON.stringify(data));
            });
          break;

        case SocketResponse.LeaveEvent:
          console.log(`User ${data.user_id} left conversation ${data.conversation_id}`);
          if (userId === data.user_id) {
            dispatch(deleteConversation(data.conversation_id));
          }
          break;

        default:
          console.log(`Unknown SocketResponseType: ${type}`);
          console.log(JSON.stringify(data));
      }
    });

    return () => {
      console.log("Cleaning up websocket connection");
      socketRef.current?.close();
    };
  }, []);

  return {
    handleSendMessage: (message: string, conversationId?: number, aiModel?: number) => {
      if (!socketRef.current) {
        console.error(`Tried to send message ${message} while WS is null`);
        return;
      }
      wsSendMessage(socketRef.current, message, conversationId, aiModel);
    },

    sendFriendRequest: (username: string, accept: boolean) => {
      if (!socketRef.current) return;
      getUserIdFromUsername(username)
        .then((id) => {
          if (!id || !socketRef.current) return;
          wsSendFriendRequest(socketRef.current, id, accept);
        })
        .catch((err) => console.log(err));
    },

    requestConversations: () => {
      if (!socketRef.current) return;
      wsRequestConversations(socketRef.current);
    },

    // inviteUsers: (usernames: string[]) => {
    inviteUsers: (friendIds: number[]) => {
      if (!socketRef.current) return;
      wsInviteUsersToConvo(socketRef.current, friendIds);
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
    requestFriendRequests: () => {
      if (!socketRef.current) return;
      wsRequestFriendRequests(socketRef.current);
    },
    leaveConversation: (conversationId: number) => {
      if (!socketRef.current) return;
      wsLeaveConversation(socketRef.current, conversationId);
    },
    loading,
  };
}
