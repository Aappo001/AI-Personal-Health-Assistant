import Background from "./Background";
import useChatSetup from "../store/hooks/useChat";

export default function WebsocketTesting() {
  const { loading } = useChatSetup();
  return (
    <>
      <Background>
        <div className="w-full flex flex-col justify-center items-center">
          <h1 className=" text-3xl text-offwhite">
            {loading
              ? "Websocket connection loading...."
              : "Websocket connection established"}
          </h1>
        </div>
      </Background>
    </>
  );
}
