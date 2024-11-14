import { useContext, useState } from "react";
import useUserStore from "../store/hooks/useUserStore";
import { WebsocketContext } from "./Chat";

export const ChatHome = () => {
  const user = useUserStore();
  const [query, setQuery] = useState("");
  //@ts-expect-error awaiting implementation
  const [selectedModel, setSelectedModel] = useState(3);
  const ws = useContext(WebsocketContext);

  const handleQueryChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setQuery(e.target.value);
  };

  const handleSubmit = (e: React.ChangeEvent<HTMLFormElement>) => {
    e.preventDefault();
    ws.handleSendMessage(query, undefined, selectedModel);
    setQuery("");
  };

  return (
    <>
      <div className="flex flex-col justify-center items-center w-screen h-screen">
        <h1 className="text-5xl text-offwhite leading-relaxed my-16">
          {user.username
            ? `Hello ${user.username}, how can I help you today`
            : "How can I help you today?"}
        </h1>
        <form
          onSubmit={handleSubmit}
          className="bg-[#363131] w-1/2 focus:outline-none rounded-full text-offwhite flex justify-between"
        >
          <input
            type="text"
            name="query"
            onChange={handleQueryChange}
            value={query}
            placeholder="Enter question"
            className="px-8 py-5 focus:outline-none bg-transparent placeholder:text-offwhite placeholder:text-lg w-5/6"
          />
          <button
            type="submit"
            className="px-8 py-5 w-32 rounded-full bg-lilac text-main-black font-bold"
          >
            Submit
          </button>
        </form>
      </div>
    </>
  );
};
