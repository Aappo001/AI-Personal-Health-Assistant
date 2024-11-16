import { useState } from "react";
import { LoginBody } from "../types";
import Background from "./Background";
import { getJwtFromResponseHeader, saveJwtToLocalStorage } from "../utils/utils";
import axios from "axios";
import useImplicitLogin from "../store/hooks/useImplicitLogin";
import useUserStore from "../store/hooks/useUserStore";
import { useNavigate } from "react-router-dom";

export default function Login() {
  useImplicitLogin();
  const userStore = useUserStore();
  const navigate = useNavigate();
  const [login, setLogin] = useState<LoginBody>({
    username: "",
    password: "",
  });
  const [responseMessage, setResponseMessage] = useState("");
  const [error, setError] = useState(false);

  if (userStore.id !== -1) {
    navigate("/");
  }

  const handleSubmit = async (event: React.ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();
    console.log(`Attempted login: ${JSON.stringify(login)}`);

    if (!login.username || !login.password) {
      setResponseMessage("Please fill in all fields.");
      setError(true);
      return;
    }

    try {
      const response = await axios.post("http://localhost:3000/api/login", login, {
        headers: {
          "Content-Type": "application/json",
        },
      });

      // Access the message directly from the response data
      setResponseMessage(response.data.message);
      setError(false);

      // Save the JWT from response headers
      const jwt = getJwtFromResponseHeader(response);

      if (!jwt) throw new Error("Internal Server Error");

      saveJwtToLocalStorage(jwt); // Save the JWT to local storage
    } catch (error: any) {
      if (error.response) {
        setResponseMessage(error.response.data.message);
      } else {
        setResponseMessage("An error occurred. Please try again later.");
      }
      setError(true);
    }
  };

  return (
    <Background color="black">
      <div className="w-full h-full flex flex-col justify-center items-center">
        <div className="border-2 border-offwhite px-4 py-4 flex flex-col items-center justify-evenly gap-4 w-3/12 h-3/5 rounded:sm">
          <p className={`text-offwhite text-6xl font-bebas`}>Login</p>
          <form
            className="flex flex-col justify-center items-center gap-9 w-5/6"
            onSubmit={handleSubmit}
          >
            <div>
              <input
                type="text"
                name="username"
                value={login.username}
                onChange={(e) => {
                  setLogin({
                    ...login,
                    username: e.target.value,
                  });
                }}
                placeholder="Username"
                autoComplete="off"
                required
                className="w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
              />
              <input
                type="password"
                value={login.password}
                name="password"
                onChange={(e) => {
                  setLogin({
                    ...login,
                    password: e.target.value,
                  });
                }}
                placeholder="Password"
                required
                className="w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
              />
            </div>
            <button
              type="submit"
              className={`px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-lilac text-lilac hover:bg-lilac hover:text-main-black`}
            >
              Submit
            </button>
          </form>
          {responseMessage && (
            <div className={`mt-4 text-xl ${error ? "text-red-500" : "text-green-500"}`}>
              {responseMessage}
            </div>
          )}
        </div>
      </div>
    </Background>
  );
}
