interface Props {
  message: string;
  from: string;
  isFromUser: boolean;
}

export default function SpeechBubble({ message, from, isFromUser }: Props) {
  const bgColor = isFromUser ? "bg-lilac" : from === "AI" ? "bg-offwhite" : "bg-orangey";
  const spacing = isFromUser ? "self-end" : "self-start";
  return (
    <div className="flex flex-col">
      <p
        className={`px-5 py-3 ${
          message.length > 125 ? "rounded-3xl" : "rounded-full"
        } text-xl ${bgColor} ${spacing}`}
      >
        {message}
      </p>
      {!isFromUser && (
        <p
          className={`${
            from === "AI" ? "text-offwhite" : "text-orangey"
          } font-semibold text-base ml-3`}
        >
          {from}
        </p>
      )}
    </div>
  );
}
