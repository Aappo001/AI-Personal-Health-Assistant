import { useContext } from "react";
import { Friend } from "../types";
import { WebsocketContext } from "./Chat";

interface Props {
  friend: Friend;
  selectedFriends: number[];
  setSelectedFriends: React.Dispatch<React.SetStateAction<number[]>>;
}
export default function FriendBox({ friend, selectedFriends, setSelectedFriends }: Props) {
  const { sendFriendRequest } = useContext(WebsocketContext);
  const isSelected = selectedFriends.some(
    (selectedFriendId) => selectedFriendId === friend.id
  );

  const handleFriendRequest = (accept: boolean) => {
    sendFriendRequest(friend.username, accept);
  };

  const handleFriendSelect = (id: number) => {
    if (selectedFriends.some((selectedFriend) => selectedFriend === id)) {
      setSelectedFriends(selectedFriends.filter((friend) => friend !== id));
      return;
    }
    setSelectedFriends((prev) => [...prev, id]);
  };

  return (
    <>
      <div
        className={`flex justify-between items-center  ${
          isSelected ? "bg-slate-700" : "bg-main-grey"
        } p-4 rounded-lg w-full`}
      >
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
          {friend.status === "Accepted" && (
            <>
              <button
                onClick={() => handleFriendSelect(friend.id)}
                className="flex justify-center items-center cursor-pointer"
              >
                <p className="text-lilac">Invite to Convo</p>
              </button>
            </>
          )}
        </div>
      </div>
    </>
  );
}
