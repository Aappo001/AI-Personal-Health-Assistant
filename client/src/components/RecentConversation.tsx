interface Props {
  friend?: string;
  recentMessage?: string;
  color?: string;
  index: number;
  onClick: (index: number) => void;
  activeIndex: number;
}
export default function RecentConversation({
  friend = "John Hancock",
  recentMessage = "No previous messages",
  color = "bg-lilac",
  index,
  onClick,
  activeIndex,
}: Props) {
  return (
    <>
      <div
        className={`flex gap-3 bg-main-grey ${
          activeIndex === index && " bg-slate-700"
        } p-4 rounded-lg w-10/12 cursor-pointer hover:scale-105`}
        onClick={() => onClick(index)}
      >
        <span className={` w-12 h-12 ${color} rounded-full`}></span>
        <div>
          <p className="text-offwhite text-xl">{friend}</p>
          <p className=" text-surface75">{recentMessage}</p>
        </div>
      </div>
    </>
  );
}
