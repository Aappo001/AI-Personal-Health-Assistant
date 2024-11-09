import { useSelector } from "react-redux";
import { Rootstate } from "../store";

export default function useMessageStore() {
  return useSelector((state: Rootstate) => state.messageState.messages);
}
