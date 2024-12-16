import { useContext, useEffect, useState } from "react";
import useFriendStore from "../store/hooks/useFriendStore";
import { WebsocketContext } from "./Chat";
import { checkUsername } from "../utils/utils";
import FriendBox from "./FriendBox";
import { useNavigate } from "react-router-dom";
import { useUserMapContext } from "./UserMapContext";

export default function FriendsPage() {
  const friendStore = useFriendStore();
  const userMap = useUserMapContext();
  const [friend, setFriend] = useState("");
  const [response, setResponse] = useState("");
  const [selectedFriends, setSelectedFriends] = useState<number[]>([]);
  const { sendFriendRequest, requestFriendRequests, inviteUsers } =
    useContext(WebsocketContext);

    const navigate = useNavigate();
  useEffect(() => {
    requestFriendRequests();
  }, []);

  useEffect(() => {
    if (friend.trim() === "") {
      setResponse("");
      return;
    }
    const timerId = setTimeout(() => {
      if (Object.values(userMap).some((user) => user.username === friend)) {
        setResponse("Already friends");
        return;
      }
      checkUsername(friend).then((usernameUnused) => {
        if (usernameUnused) {
          setResponse("User doesn't exist");
        } else {
          setResponse("User Exists");
        }
      });
    }, 450);

    return () => {
      clearInterval(timerId);
    };
  }, [friend]);

  const createConversation = () => {
    inviteUsers(selectedFriends);
  };

  return (
    <>
      <div className="w-screen h-screen flex flex-col justify-center items-center">
      <div className="flex justify-end p-4">
          <button
            onClick={() => navigate("/")}
            className="bg-blue-500 text-white px-4 py-2 rounded-md shadow-md hover:bg-blue-600"
          >
            Home
          </button>
        </div>
        {response && <p className="text-xl text-offwhite">{response}</p>}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setResponse("Friend request sent!");
            sendFriendRequest(friend, true);
          }}
          className="bg-[#363131] w-1/4 focus:outline-none rounded-full text-offwhite flex justify-between"
        >
          <input
            type="text"
            value={friend}
            onChange={(e) => setFriend(e.target.value)}
            placeholder="Add friend"
            className="px-8 py-5 focus:outline-none bg-transparent placeholder:text-offwhite placeholder:text-lg w-5/6"
          />
          <button
            type="submit"
            className="px-8 py-5 w-32 rounded-full bg-lilac text-main-black font-bold"
          >
            Add
          </button>
        </form>
        {selectedFriends.length > 0 && (
          <>
            <button
              onClick={createConversation}
              className="w-1/4 bg-lilac rounded-full px-5 py-3 font-bold text-xl mt-4"
            >
              Start Conversation!
            </button>
          </>
        )}
        <div className="flex flex-col justify-center items-center gap-4 w-1/6 mt-4">
          {friendStore.map((friend, i) => (
            <FriendBox
              friend={friend}
              selectedFriends={selectedFriends}
              setSelectedFriends={setSelectedFriends}
              key={`${friend.username}-${i}`}
            />
          ))}
        </div>
      </div>
    </>
  );
}
