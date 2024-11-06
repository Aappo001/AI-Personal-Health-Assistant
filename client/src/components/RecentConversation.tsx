import { Friend } from "../types";

interface Props {
  friend: Friend;
  recentMessage?: string;
  onClick: (index: number) => void;
  activeIndex: number;
}
export default function RecentConversation({
  friend,
  recentMessage = "No previous messages",
  onClick,
  activeIndex,
}: Props) {
  return (
    <>
      <div
        className={`flex gap-3 bg-main-grey ${
          activeIndex === friend.id && " bg-slate-700"
        } p-4 rounded-lg w-10/12 cursor-pointer hover:scale-105`}
        onClick={() => onClick(friend.id)}
      >
        <span className={` w-12 h-12 ${friend.color} rounded-full`}></span>
        <div>
          <p className="text-offwhite text-xl">{friend.username}</p>
          <p className=" text-surface75">{recentMessage}</p>
        </div>
      </div>
    </>
  );
}
