import { useState } from "react";
import { RegisterBody } from "../types";

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

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUser({ ...user, [event.target.name]: event.target.value });
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
      const response = await fetch("http://localhost:3000/api/register", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(user),
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
            type="email"
            value={user.email}
            name="email"
            onChange={handleChange}
            placeholder="Email"
            required
            className="p-2 mb-2 border border-gray-300 rounded"
          />
          <input
            type="text"
            value={user.firstName}
            name="firstName"
            onChange={handleChange}
            placeholder="First Name"
            required
            className="p-2 mb-2 border border-gray-300 rounded"
          />
          <input
            type="text"
            value={user.lastName}
            name="lastName"
            onChange={handleChange}
            placeholder="Last Name"
            required
            className="p-2 mb-2 border border-gray-300 rounded"
          />
          <input
            type="text"
            name="username"
            value={user.username}
            onChange={handleChange}
            placeholder="Username"
            required
            className="p-2 mb-2 border border-gray-300 rounded"
          />
          <input
            type="password"
            value={user.password}
            name="password"
            onChange={handleChange}
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
