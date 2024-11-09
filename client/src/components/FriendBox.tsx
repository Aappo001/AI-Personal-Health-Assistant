import { useContext } from "react";
import { Friend } from "../types";
import { WebsocketContext } from "./Chat";

interface Props {
  friend: Friend;
}
export default function FriendBox({ friend }: Props) {
  const { sendFriendRequest } = useContext(WebsocketContext);

  const handleFriendRequest = (accept: boolean) => {
    sendFriendRequest(friend.username, accept);
  };

  return (
    <>
      <div className="flex justify-between items-center bg-main-grey p-4 rounded-lg w-full">
        <div className={`flex gap-3 `}>
          <span className={` w-12 h-12 ${friend.color} rounded-full`}></span>
          <div>
            <p className="text-offwhite text-xl">{friend.username}</p>
            <p className=" text-surface75">
              {friend.status === "Accepted"
                ? "Friends"
                : friend.isSender
                ? "Pending"
                : "Pending (Outgoing)"}
            </p>
          </div>
          {friend.status === "Pending" && friend.isSender === true && (
            <div className=" flex gap-4">
              <button onClick={() => handleFriendRequest(true)} className="text-main-green">
                Accept
              </button>
              <button onClick={() => handleFriendRequest(false)} className=" text-red-700">
                Deny
              </button>
            </div>
          )}
        </div>
      </div>
    </>
  );
}
