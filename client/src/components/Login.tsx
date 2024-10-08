import { useState } from "react";
import { LoginBody } from "../types";

export default function Login() {
  const [login, setLogin] = useState<LoginBody>({
    username: "",
    password: "",
  });
  const [responseMessage, setResponseMessage] = useState("");
  const [error, setError] = useState(false);

  const handleSubmit = async (event: React.ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();
    console.log(`Attempted login: ${JSON.stringify(login)}`);

    if (!login.username || !login.password) {
      setResponseMessage("Please fill in all fields.");
      setError(true);
      return;
    }

    try {
      const response = await fetch("http://localhost:3000/api/login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(login),
      });

      const result = await response.json();
      if (response.ok) {
        setResponseMessage(result.message);
        setError(false);
      } else {
        setResponseMessage(result.message);
        setError(true);
      }
    } catch (error) {
      setResponseMessage("An error occurred. Please try again later.");
      setError(true);
    }
  };

  return (
    <>
      <div className="flex flex-col items-center justify-center h-screen bg-gray-200">
        <h2 className="text-2xl font-bold mb-4">Login</h2>
        <form onSubmit={handleSubmit} className="flex flex-col w-72 m-5">
          <input
            type="text"
            value={login.username}
            onChange={(e) => setLogin({ ...login, username: e.target.value })}
            placeholder="Username"
            required
            className="p-2 mb-2 border border-gray-300 rounded"
          />
          <input
            type="password"
            value={login.password}
            onChange={(e) => setLogin({ ...login, password: e.target.value })}
            placeholder="Password"
            required
            className="p-2 mb-2 border border-gray-300 rounded"
          />
          <button
            type="submit"
            className="p-2 bg-green-500 text-white rounded hover:bg-green-600"
          >
            Login
          </button>
        </form>
        {responseMessage && (
          <div
            className={`mt-4 text-xl ${
              error ? "text-red-500" : "text-green-500"
            }`}
          >
            {responseMessage}
          </div>
        )}
      </div>
    </>
  );
}
