import { useContext, useRef, useState } from "react";
import useUserStore from "../store/hooks/useUserStore";
import { WebsocketContext } from "./Chat";
import FileAttachment from "./FileAttachment";
import { Attachment } from "../types";
import { uploadAttachment } from "../utils/utils";

export const ChatHome = () => {
  const user = useUserStore();
  const [query, setQuery] = useState("");
  //@ts-expect-error awaiting implementation
  const [selectedModel, setSelectedModel] = useState(3);
  const inputRef = useRef<HTMLInputElement>(null);
  const [inputFile, setInputFile] = useState<Attachment>({ file_name: "", file_data: "" });
  const ws = useContext(WebsocketContext);

  const handleQueryChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setQuery(e.target.value);
  };

  const handleSubmit = (e: React.ChangeEvent<HTMLFormElement>) => {
    e.preventDefault();
    ws.handleSendMessage(query, undefined, selectedModel);
    if (inputFile.file_data) {
      uploadAttachment(inputFile);
    }
    setQuery("");
  };

  const handleFileUploadClick = () => {
    if (!inputRef.current) {
      throw new Error("Input ref is null, idk how this happened");
    }
    inputRef.current.click();
  };

  const handleFileClear = () => {
    if (!inputRef.current) {
      throw new Error("Input ref is null, idk how this happened");
    }
    inputRef.current.value = "";
    setInputFile({ file_name: "", file_data: "" });
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    console.log(JSON.stringify(e.target.files?.[0]));

    if (!file) return;
    const reader = new FileReader();
    reader.onloadend = () => {
      console.log(`Result: ${reader.result}`);
      setInputFile({ file_name: file.name, file_data: reader.result as string });
    };

    reader.readAsDataURL(file);
  };

  return (
    <>
      <div className="flex flex-col justify-center items-center w-screen h-screen">
        <h1 className="text-5xl text-offwhite leading-relaxed my-16">
          {user.username
            ? `Hello ${user.username}, how can I help you today`
            : "How can I help you today?"}
        </h1>
        {inputFile.file_name && (
          <FileAttachment fileName={inputFile.file_name} handleFileClear={handleFileClear} />
        )}
        <form
          onSubmit={handleSubmit}
          className="bg-[#363131] w-1/2 focus:outline-none rounded-full text-offwhite flex justify-between"
        >
          <input type="file" className=" hidden" ref={inputRef} onChange={handleFileChange} />
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
