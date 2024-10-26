import { useEffect, useRef, useState } from "react";
import { getJwt, getUserIdFromUsername } from "../../utils/utils";
import { wsSendFriendRequest, SocketResponse } from "../../utils/ws-utils";

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
    default:
      console.log(`Unknown SocketResponseType: ${type}`);
  }

  console.log(JSON.stringify(data));
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
    handleSendMessage: (message: string) => {
      if (!socketRef.current) {
        console.error(`Tried to send message ${message} while WS is null`);
        return;
      }
      socketRef.current.send(
        JSON.stringify({
          type: "SendMessage",
          message: message,
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

    loading,
  };
}
