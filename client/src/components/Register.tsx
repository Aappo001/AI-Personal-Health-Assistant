import { useCallback, useState } from "react";
import { RegisterBody } from "../types";
import Background from "./Background";
import axios from "axios";
import { useNavigate } from "react-router-dom";
import { BASE_URL, RegisterUser, debounce } from "../utils/utils";

export default function Register() {
  const [user, setUser] = useState<RegisterBody>({
    firstName: "",
    lastName: "",
    username: "",
    email: "",
    password: "",
  });
  const [responseMessage, setResponseMessage] = useState("");
  const [error, setError] = useState(false);

  const navigate = useNavigate();
  const debounceCheck = useCallback(debounce(async (event: React.ChangeEvent<HTMLInputElement>) => {
    if ((event.target.name === "username" || event.target.name === "email") && event.target.value) {
      try {
        const response = await axios.get(`${BASE_URL}/api/check/${event.target.name}/${event.target.value}`); // Use Axios for the API request

        if (response.status !== 200) {
          let error = response.data;
          setResponseMessage(error.message);
          setError(true);
        } else {
          setResponseMessage(`Valid ${event.target.name}`);
          setError(false);
        }
      } catch (error) {
        setResponseMessage("An error occurred. Please try again later.");
        setError(true);
      }
    }
  }, 700), []);

  const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    setUser({ ...user, [event.target.name]: event.target.value });
    debounceCheck(event)
  };

  const handleSubmit = async (event: React.ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();
    console.log(`Attempted Register: ${JSON.stringify(user)}`);

    if (
      !user.username ||
      !user.password ||
      !user.firstName ||
      !user.lastName ||
      !user.email
    ) {
      setResponseMessage("Please fill in all fields.");
      setError(true);
      return;
    }

    try {
      const result = await RegisterUser(user);
      if ("errorMessage" in result) {
        setError(true);
        setResponseMessage(result.errorMessage);
      } else {
        setError(false);
        setResponseMessage(result.message);
      }
    } catch (error) {
      setResponseMessage("An error occurred. Please try again later.");
      setError(true);
    }
  };

  return (
    <Background color="black">
            <div className="flex justify-end p-4">
          <button
            onClick={() => navigate("/")}
            className="bg-blue-500 text-white px-4 py-2 rounded-md shadow-md hover:bg-blue-600"
          >
            Home
          </button>
        </div>
      <div className="w-full h-full flex flex-col justify-center items-center">
        <div className="border-2 border-offwhite px-4 py-4 flex flex-col items-center justify-evenly gap-4 w-3/12 h-4/5 rounded:sm">
          <p className={`text-offwhite text-6xl font-bebas`}>Register</p>
          <form
            className="flex flex-col justify-center items-center gap-9 w-5/6"
            onSubmit={handleSubmit}
          >
            <div>
              <input
                type="email"
                value={user.email}
                name="email"
                onChange={handleChange}
                placeholder="Email"
                required
                className="w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
              />
              <input
                type="text"
                value={user.firstName}
                name="firstName"
                onChange={handleChange}
                placeholder="First Name"
                required
                className="w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
              />
              <input
                type="text"
                value={user.lastName}
                name="lastName"
                onChange={handleChange}
                placeholder="Last Name"
                required
                className="w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
              />
              <input
                type="text"
                name="username"
                value={user.username}
                onChange={handleChange}
                placeholder="Username"
                required
                className="w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-lilac bg-main-black text-offwhite"
              />
              <input
                type="password"
                value={user.password}
                name="password"
                onChange={handleChange}
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
            <div
              className={`mt-4 text-xl ${error ? "text-red-500" : "text-green-500"
                }`}
            >
              {responseMessage}
            </div>
          )}
        </div>
      </div>
    </Background>
  );
}
