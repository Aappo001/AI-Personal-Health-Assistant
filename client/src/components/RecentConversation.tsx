import { Friend } from "../types";

interface Props {
  // friend: Friend;
  id: number;
  recentMessage?: string;
  onClick: (index: number) => void;
  activeIndex: number;
}
export default function RecentConversation({
  // friend,
  id,
  recentMessage = "No previous messages",
  onClick,
  activeIndex,
}: Props) {
  return (
    <>
      <div
        className={`flex gap-3 bg-main-grey ${
          activeIndex === id && " bg-slate-700"
        } p-4 rounded-lg w-10/12 cursor-pointer hover:scale-105`}
        onClick={() => onClick(id)}
      >
        {/* <span className={` w-12 h-12 ${friend.color} rounded-full`}></span> */}
        <span className={` w-12 h-12 bg-lilac rounded-full`}></span>
        <div>
          {/* <p className="text-offwhite text-xl">{friend.username}</p> */}
          <p className="text-offwhite text-xl">Conversation {id}</p>
          <p className=" text-surface75">{recentMessage}</p>
        </div>
      </div>
    </>
  );
}
