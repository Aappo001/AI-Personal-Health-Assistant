import DOMPurify from "dompurify";
import { marked } from "marked";
import { useEffect, useState } from "react";

interface Props {
  message: string;
  from: string;
  isFromUser: boolean;
}

export default function SpeechBubble({ message, from, isFromUser }: Props) {
  const bgColor = isFromUser ? "bg-lilac" : from === "AI" ? "bg-offwhite" : "bg-orangey";
  const spacing = isFromUser ? "self-end" : "self-start";
  const [renderedMessage, setRenderedMessage] = useState("");
  useEffect(() => {
    const inner = async () => {
      const rendered = await marked.parse(message);
      setRenderedMessage(rendered);
    }
    inner()
  }, [message]);
  return (
    <div className="speechBubble flex flex-col">
      <div
        className={`px-5 py-3 ${message.length > 125 ? "rounded-3xl" : "rounded-full"
          } text-xl ${bgColor} ${spacing}`}
        dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(renderedMessage) }} />
      {!isFromUser && (
        <p
          className={`${from === "AI" ? "text-offwhite" : "text-orangey"
            } font-semibold text-base ml-3`}
        >
          {from}
        </p>
      )}
    </div>
  );
}
