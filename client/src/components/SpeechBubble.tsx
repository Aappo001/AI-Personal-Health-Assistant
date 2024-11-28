import DOMPurify from "dompurify";
import { marked } from "marked";
import { Children, useEffect, useState } from "react";

interface Props {
  message: string;
  from: string;
  isFromUser: boolean;
  children?: React.ReactNode;
}

export default function SpeechBubble({ message, from, isFromUser, children }: Props) {
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
        className={`px-5 py-3 ${children ? "rounded-lg" : (message.length > 125 ? "rounded-3xl" : "rounded-full")
          } text-xl ${bgColor} ${spacing}`}>
        <div className="flex flex-col space-y-2">
          {Children.map(children, (child) =>
            (<div className="w-full h-auto my-1">{child}</div>)
          )}
          <div dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(renderedMessage) }} />
        </div>
      </div>
      {
        !isFromUser && (
          <p
            className={`${from === "AI" ? "text-offwhite" : "text-orangey"
              } font-semibold text-base ml-3`}
          >
            {from}
          </p>
        )
      }
    </div >
  );
}
