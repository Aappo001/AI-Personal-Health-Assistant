import { useSelector } from "react-redux";
import { Rootstate } from "../store";

export default function useFriendStore() {
  return useSelector((state: Rootstate) => state.friendsState.friends);
}
