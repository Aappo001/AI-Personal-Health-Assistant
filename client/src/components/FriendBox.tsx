import { Friend } from "../types";

interface Props {
  friend: Friend;
}
export default function FriendBox({ friend }: Props) {
  return (
    <>
      <div className={`flex gap-3 bg-main-grey p-4 rounded-lg w-full`}>
        <span className={` w-12 h-12 ${friend.color} rounded-full`}></span>
        <div>
          <p className="text-offwhite text-xl">{friend.username}</p>
          <p className=" text-surface75">Friends</p>
        </div>
      </div>
    </>
  );
}
