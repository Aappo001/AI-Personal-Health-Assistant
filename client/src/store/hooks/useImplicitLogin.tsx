import { loginImplicitly } from "../../utils/utils";

export default function useImplicitLogin() {
  let user = "";
  loginImplicitly()
    .then((result) => {
      if (!result) return;
      user = result;
    })
    .catch((err) => {
      console.log(`Implicit Login Error: ${err}`);
    });
  return user;
}
