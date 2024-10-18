import { useEffect } from "react";
import { loginImplicitly } from "../../utils/utils";
import useAppDispatch from "./useAppDispatch";
import { updateUser } from "../userSlice";

export default function useImplicitLogin() {
  const dispatch = useAppDispatch();

  useEffect(() => {
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
  }, []);
}
