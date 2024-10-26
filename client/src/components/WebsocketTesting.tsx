import Background from "./Background";
import useWebsocketSetup from "../store/hooks/useWebsocket";

export default function WebsocketTesting() {
  const { handleSendMessage, sendFriendRequest, loading } = useWebsocketSetup();
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
              sendFriendRequest("kkk");
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Send Friend Req to username kkk
          </button>
          <button
            onClick={() => {
              sendFriendRequest("jjj");
            }}
            className="px-8 py-3 border-2 border-lilac font-bold rounded-full text-lilac transition-colors duration-200 hover:bg-lilac hover:text-black"
          >
            Accept Friend Req to username jjj
          </button>
        </div>
      </Background>
    </>
  );
}
