import { useSelector } from "react-redux";
import { Rootstate } from "../store";

export default function useConversationStore() {
  return useSelector((state: Rootstate) => state.messageState.conversations);
}
