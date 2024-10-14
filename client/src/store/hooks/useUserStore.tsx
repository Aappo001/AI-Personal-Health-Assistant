import { useSelector } from "react-redux";
import { Rootstate } from "../store";
export default function useUserStore() {
  return useSelector((state: Rootstate) => state.user);
}
