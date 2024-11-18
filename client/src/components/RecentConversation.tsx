import { useContext } from "react";
import { WebsocketContext } from "./Chat";
import { useNavigate } from "react-router-dom";

interface Props {
  id: number;
  title: string | undefined;
  recentMessage?: string;
  onClick: (index: number) => void;
  activeIndex: number;
}
export default function RecentConversation({
  id,
  title = `Conversation ${id}`,
  recentMessage = "No previous messages",
  onClick,
  activeIndex,
}: Props) {
  const { leaveConversation } = useContext(WebsocketContext);
  const navigate = useNavigate();

  return (
    <>
      <div className="flex w-10/12 hover:scale-105">
        <div
          className={`flex gap-3 bg-main-grey ${
            activeIndex === id && " bg-slate-700"
          } p-4 rounded-lg rounded-r-none w-10/12 cursor-pointer `}
          onClick={() => onClick(id)}
        >
          <span className={` w-12 h-12 bg-lilac rounded-full flex-shrink-0`}></span>
          <div>
            <p className="text-offwhite text-xl">{title}</p>
            <p className=" text-surface75">{recentMessage}</p>
          </div>
        </div>
        <div
          className={`bg-main-grey flex justify-center items-center flex-shrink-0 ${
            activeIndex === id && " bg-slate-700"
          } p-4 rounded-lg rounded-l-none cursor-pointer `}
        >
          <img
            src="/exit.svg"
            height={25}
            width={25}
            onClick={() => {
              leaveConversation(id);
              navigate("/chat");
            }}
          />
        </div>
      </div>
    </>
  );
}
