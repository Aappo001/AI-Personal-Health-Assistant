import axios, { AxiosResponse } from "axios";
import { implicitLoginSchema } from "../schemas";
import { RegisterBody, ServerResponse, ErrorResponse, SessionUser } from "../types";

export async function RegisterUser(
  user: RegisterBody
): Promise<ServerResponse | ErrorResponse> {
  try {
    const response = await axios.post<ServerResponse>("http://localhost:3000/api/register", user, {
      headers: {
        "Content-Type": "application/json",
      },
    });

    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error) && error.response) {
      return { errorMessage: error.response.data.message };
    }
    return { errorMessage: "An error occurred. Please try again later." };
  }
}

export function debounce<T extends unknown[]>(func: (...args: T) => void, delay: number):
  (...args: T) => void {
  let timer: number | null = null;
  return (...args: T) => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      func.call(null, ...args);
    }, delay);
  };
}

export const loginImplicitly = async (): Promise<SessionUser | undefined> => {
  const jwt = getJwt();
  if (!jwt) return;
  
  try {
    const response = await axios.get<SessionUser>("http://localhost:3000/api/login", {
      headers: {
        "Content-Type": "application/json",
        "authorization": `Bearer ${jwt}`,
      },
    });

    const parsedData = implicitLoginSchema.safeParse(response.data);
    if (parsedData.error) {
      console.log(parsedData.error);
      return;
    }
    console.log(`Successful Implicit Login: User ${parsedData.data.username}`);
    return parsedData.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.log("Implicit Login Error", error.response?.data);
    }
    return;
  }
};

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

export const getJwtFromResponseHeader = (response: AxiosResponse) => {
  const token = response.headers['authorization']?.split(" ")[1]
  if (!token) return "";
  return token;
};

export const saveJwtToLocalStorage = (jwt: string) => {
  localStorage.setItem("token", jwt);
};

export const getJwt = (): string => {
  const jwt = localStorage.getItem("token");
  if (!jwt) return "";
  return jwt;
};
