import { useEffect, useRef, useState } from "react";
import { getJwt } from "../../utils/utils";
import { userChatMessageSchema } from "../../schemas";

const handleMessage = (event: MessageEvent) => {
  console.log("Received websocket response");
  const data = JSON.parse(event.data);
  console.log(JSON.stringify(data));

  const parsedMessage = userChatMessageSchema.safeParse(data);
  if (!parsedMessage.success) {
    console.log("Error parsing websocket response");
    return;
  }
};

export default function useChatSetup() {
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
          message: "Hello vro",
        })
      );
    },
    loading,
  };
}
