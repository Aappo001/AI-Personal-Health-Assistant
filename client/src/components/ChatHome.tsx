import { useContext, useState } from "react";
import useUserStore from "../store/hooks/useUserStore";
import { WebsocketContext } from "./Chat";
import FileAttachment from "./FileAttachment";
import { uploadAttachment } from "../utils/utils";
import useFileAttachment from "../store/hooks/useFileAttachment";
import { UploadAttachment } from "../types";
import { useNavigate } from "react-router-dom";

export const ChatHome = () => {
  const user = useUserStore();
  const [query, setQuery] = useState("");
  //@ts-expect-error awaiting implementation
  const [selectedModel, setSelectedModel] = useState(3);
  const ws = useContext(WebsocketContext);
  const { attachment, hiddenFileInput, handleFileUploadClick, resetFile } =
    useFileAttachment();
  const navigate = useNavigate();

  const handleQueryChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setQuery(e.target.value);
  };

  const handleSubmit = async (e: React.ChangeEvent<HTMLFormElement>) => {
    e.preventDefault();
    let file_id: number | undefined;
    let messageAttachment: UploadAttachment | undefined;
    if (attachment.fileData) {
      file_id = await uploadAttachment(attachment);
      messageAttachment = {
        id: file_id,
        name: attachment.fileName,
      };
    }
    ws.handleSendMessage(query, undefined, selectedModel, messageAttachment);
    setQuery("");
  };

  return (
    <>
          <div className="flex justify-end p-4">
          <button
            onClick={() => navigate("/")}
            className="bg-blue-500 text-white px-4 py-2 rounded-md shadow-md hover:bg-blue-600"
          >
            Home
          </button>
        </div>
      <div className="flex flex-col justify-center items-center w-screen h-screen">
        <h1 className="text-5xl text-offwhite leading-relaxed my-16">
          {user.username
            ? `Hello ${user.username}, how can I help you today`
            : "How can I help you today?"}
        </h1>
        {attachment.fileName && (
          <FileAttachment fileName={attachment.fileName} handleFileClear={resetFile} />
        )}
        <form
          onSubmit={handleSubmit}
          className="bg-[#363131] w-1/2 focus:outline-none rounded-full text-offwhite flex justify-between"
        >
          {hiddenFileInput()}
          <img
            src="/plus-circle.svg"
            className="ml-3 cursor-pointer"
            alt="Add File"
            height={35}
            width={35}
            onClick={handleFileUploadClick}
          />
          <input
            type="text"
            name="query"
            onChange={handleQueryChange}
            value={query}
            placeholder="Enter question"
            className="px-8 py-5 pl-4 focus:outline-none bg-transparent placeholder:text-offwhite placeholder:text-lg w-5/6"
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
