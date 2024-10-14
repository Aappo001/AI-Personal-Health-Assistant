interface Props {
  friend?: string;
  recentMessage?: string;
}
export default function RecentConversation({
  friend = "John Hancock",
  recentMessage = "No previous messages",
}: Props) {
  const mainColors = [
    "bg-main-green",
    "bg-orangey",
    "bg-lilac",
    "bg-main-blue",
    "bg-shock-pink",
  ];
  const randomColor = () => {
    return mainColors[Math.floor(Math.random() * mainColors.length)];
  };

  return (
    <>
      <div className="flex gap-3 bg-main-grey p-4 rounded-lg w-10/12 cursor-pointer hover:scale-105">
        <span className={` w-12 h-12 ${randomColor()} rounded-full`}></span>
        <div>
          <p className="text-offwhite text-xl">{friend}</p>
          <p className=" text-surface75">{recentMessage}</p>
        </div>
      </div>
    </>
  );
}
