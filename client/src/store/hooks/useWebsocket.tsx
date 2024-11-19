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
  wsRenameConversation,
} from "../../utils/ws-utils";
import { requestFriendsSchema } from "../../schemas";
import useAppDispatch from "./useAppDispatch";
import { addFriend, removeFriend, upgradeFriendStatus } from "../friendsSlice";
import {
  deleteConversation,
  initializeConversation,
  pushMessage,
  pushStreamMessage,
  cancelStream,
  updateTitle,
} from "../conversationSlice";
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

      switch (type) {
        case SocketResponse.Message:
          console.log(`Received message from ${data.userId ? `User ${data.userId}` : "AI"}`);

          // This only throws an error if the conversation is not initialized
          try {
            dispatch(
              pushMessage({
                id: data.conversationId,
                message: {
                  userId: data.userId,
                  content: data.message,
                  fromAi: !data.userId,
                  streaming: false,
                },
              })
            );
          } catch (err) {
            // TypeError denotes that the conversation is not initialized
            if (err instanceof TypeError) {
              wsRequestConversation(socketRef.current, data.conversationId);
              dispatch(
                pushMessage({
                  id: data.conversationId,
                  message: {
                    userId: data.userId,
                    content: data.message,
                    fromAi: !data.userId,
                    streaming: false,
                  },
                })
              );
            } else {
              throw err;
            }
          }
          break;

        case SocketResponse.StreamData:
          dispatch(
            pushStreamMessage({
              id: data.conversationId,
              message: data.message,
              querierId: data.querierId,
            })
          );
          break;

        case SocketResponse.FriendRequest:
          console.log("SocketResponse: FriendRequest");
          const userIsSender = data.senderId === userId;
          let id = userIsSender ? data.receiverId : data.senderId;

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
          console.log(`Received Invite Message from User ${data.inviter}`);
          dispatch(initializeConversation({ id: data.conversationId }));
          break;

        case SocketResponse.Conversation:
          console.log(`User present in conversation with id ${data.id}`);

          dispatch(initializeConversation({ id: data.id, title: data.title }));
          wsRequestMessages(socketRef.current, data.id);
          break;

        case SocketResponse.Error:
          console.log(`SocketResponse Error: ${data.message}`);
          break;

        case SocketResponse.FriendData:
          console.log(`Friends with user id ${data.id} at ${data.createdAt}`);
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
          console.log(`User ${data.userId} left conversation ${data.conversationId}`);
          if (userId === data.userId) {
            dispatch(deleteConversation(data.conversationId));
          }
          break;

        case SocketResponse.CanceledGeneration:
          console.log(
            `User ${data.querierId} cancelled ai generation in conversation ${data.conversationId}`
          );
          dispatch(cancelStream({ id: data.conversationId, querierId: data.querierId }));
          break;

        // {"type":"RenameEvent","conversationId":1,"userId":1,"name":"george flowberry"}
        case SocketResponse.RenameEvent:
          console.log(
            `Successfully renamed Conversation ${data.conversationId} to ${data.name}`
          );
          dispatch(updateTitle({ id: data.conversationId, title: data.name }));
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
    renameConversation: (conversationId: number, name: string) => {
      if (!socketRef.current) return;
      wsRenameConversation(socketRef.current, conversationId, name);
    },
    loading,
  };
}
