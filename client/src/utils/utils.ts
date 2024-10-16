import { RegisterBody, ServerResponse, ErrorResponse } from "../types";

export async function RegisterUser(
  user: RegisterBody
): Promise<ServerResponse | ErrorResponse> {
  const response = await fetch("http://localhost:3000/api/register", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(user),
  });

  const result: ServerResponse = await response.json();
  if (!response.ok) return { errorMessage: result.message };
  console.log(response.headers);
  
  return result;
}

const mainColors = [
  "bg-main-green",
  "bg-orangey",
  "bg-lilac",
  "bg-main-blue",
  "bg-shock-pink",
];

export const generateRandomColorArray = (length: number): string[] => {
  const convoColors: string[] = [];
  for (let i = 0; i < length; i++) {
    convoColors.push(mainColors[Math.floor(Math.random() * mainColors.length)]);
  }
  return convoColors;
};

export const getJwtFromResponseHeader = (response: Response) => {
  const token = response.headers.get("authorization")?.split(" ")[1]
  if(!token) return ""
  return token
}

export const saveJwtToLocalStorage = (jwt: string) => {
  localStorage.setItem("token", jwt)
}
