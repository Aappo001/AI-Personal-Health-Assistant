import { useState } from "react";
import useUserStore from "../store/hooks/useUserStore";
import Background from "./Background";
import { LoginBody, RegisterBody } from "../types";
import { getJwt } from "../utils/utils";

export default function ProfilePage() {
  const userStore = useUserStore();

  const [user, setUser] = useState<RegisterBody>({
    firstName: userStore.firstName,
    lastName: userStore.lastName,
    username: userStore.username,
    email: userStore.email,
    password: "",
  });

  const [deleteUser, setDeleteUser] = useState<LoginBody>({
    username: "",
    password: "",
  });

  const [error, setError] = useState("");

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUser({ ...user, [event.target.name]: event.target.value });
  };

  const handleDeleteProfile = async (
    event: React.ChangeEvent<HTMLFormElement>
  ) => {
    event.preventDefault();
    try {
      const response = await fetch("http://localhost:3000/api/account", {
        method: "DELETE",
        headers: {
          "Content-Type": "application/json",
          authorization: `Bearer ${getJwt()}`,
        },
        body: JSON.stringify(deleteUser),
      });
      const result = await response.text();
      if (result === "User deleted") {
        //using window.location forces a refresh, automatically clearing the redux state
        //might want to change this in the future, but it seems logical for now
        window.location.href = "/";
      }
      setError(result);
    } catch (err) {
      console.log(`Error occurred when deleting acc: ${err}`);
    }
  };

  return (
    <Background>
      <div className="w-screen flex justify-center items-center py-16 gap-4">
        <div className="border-2 border-offwhite px-4 py-6 flex flex-col items-center justify-evenly gap-4 w-3/12 h-4/5 rounded:sm">
          <p className={` text-offwhite text-6xl font-bebas`}>Edit Profile</p>
          <form className="flex flex-col justify-center items-center gap-9 w-5/6">
            <div>
              <input
                type="email"
                value={user.email}
                name="email"
                placeholder="Email"
                onChange={handleChange}
                required
                className="w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-main-blue bg-main-black text-offwhite text-xl"
              />
              <input
                type="text"
                value={user.firstName}
                name="firstName"
                placeholder="First Name"
                onChange={handleChange}
                required
                className="text-xl w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-main-blue bg-main-black text-offwhite"
              />
              <input
                type="text"
                value={user.lastName}
                name="lastName"
                placeholder="Last Name"
                onChange={handleChange}
                required
                className="text-xl w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-main-blue bg-main-black text-offwhite"
              />
              <input
                type="text"
                name="username"
                value={user.username}
                placeholder="Username"
                onChange={handleChange}
                required
                className="text-xl w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-main-blue bg-main-black text-offwhite"
              />
              <input
                type="password"
                name="password"
                value={user.password}
                placeholder="Password"
                onChange={handleChange}
                required
                className="text-xl w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-main-blue bg-main-black text-offwhite"
              />
            </div>
            <button
              type="submit"
              className={`px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-main-blue text-main-blue hover:bg-main-blue hover:text-main-black`}
            >
              Submit
            </button>
          </form>
        </div>
        <div className="border-2 border-offwhite px-4 py-6 flex flex-col items-center justify-evenly gap-4 w-3/12 h-4/5 rounded:sm">
          <p className={` text-offwhite text-6xl font-bebas`}>Delete Profile</p>
          <form
            onSubmit={handleDeleteProfile}
            className="flex flex-col justify-center items-center gap-9 w-5/6"
          >
            <div>
              <input
                type="text"
                name="username"
                value={deleteUser.username}
                onChange={(e) => {
                  setDeleteUser({ ...deleteUser, username: e.target.value });
                }}
                placeholder="Username"
                required
                className="text-xl w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-shock-pink bg-main-black text-offwhite"
              />
              <input
                type="password"
                name="password"
                value={deleteUser.password}
                onChange={(e) => {
                  setDeleteUser({ ...deleteUser, password: e.target.value });
                }}
                placeholder="Password"
                required
                className="text-xl w-full mt-8 pr-4 py-2 border-b-[1px] placeholder:text-surface75 focus:outline-none transition-colors duration-200 border-b-offwhite focus:border-b-shock-pink bg-main-black text-offwhite"
              />
            </div>
            <button
              type="submit"
              className={`px-5 py-3 border-2 rounded-full font-bold w-full transition-colors border-shock-pink text-shock-pink hover:bg-shock-pink hover:text-main-black`}
            >
              Delete Forever
            </button>
          </form>
        </div>
      </div>

      {error && (
        <div className="w-full flex justify-center">
          <h1 className="text-2xl text-red-600">{error}</h1>
        </div>
      )}
    </Background>
  );
}
