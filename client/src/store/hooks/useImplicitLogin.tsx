import { useEffect } from "react";
import { loginImplicitly } from "../../utils/utils";
import useAppDispatch from "./useAppDispatch";
import { updateUser } from "../userSlice";
import useUserStore from "./useUserStore";

export default function useImplicitLogin() {
  const userStore = useUserStore();
  const dispatch = useAppDispatch();

  useEffect(() => {
    if (userStore.id !== -1) return;
    const login = async () => {
      try {
        const user = await loginImplicitly();
        if (user) {
          dispatch(updateUser(user));
        }
      } catch (err) {
        console.log(`Implicit Login Error: ${err}`);
      }
    };

    login();
  }, [userStore]);
}
