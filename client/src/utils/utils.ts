import axios, { AxiosResponse } from "axios";
import { implicitLoginSchema, publicUserSchema } from "../schemas";
import { RegisterBody, ServerResponse, ErrorResponse, SessionUser, PublicUser, FileUpload } from "../types";

export const BASE_URL = import.meta.env.DEV ? "http://localhost:3000" : "";

export async function RegisterUser(
  user: RegisterBody
): Promise<ServerResponse | ErrorResponse> {
  try {
    const response = await axios.post<ServerResponse>(`${BASE_URL}/api/register`, user, {
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

export async function checkUsername(username: string): Promise<boolean> {
  const response = await fetch(`${BASE_URL}/api/check/username/${username}`)
  return response.status === 200
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
    const response = await axios.get<SessionUser>(`${BASE_URL}/api/login`, {
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

export const getUserIdFromUsername = async (username: string): Promise<number | undefined> => {
  const response = await fetch(`${BASE_URL}/api/users/username/${username}`);

  if (!response.ok) {
    console.log("Error getting user id from username");
    return;
  }

  const parsedUser = publicUserSchema.safeParse(await response.json());
  if (!parsedUser.success || parsedUser.error) {
    console.log("Error sending friend request: Bad JSON received from server");
    return;
  }

  return parsedUser.data.id
}

export const getUserFromId = async (id: number): Promise<PublicUser | undefined> => {
  const response = await fetch(`${BASE_URL}/api/users/id/${id}`)
  if (!response.ok) {
    console.error("Error getting user from id");
    return
  }

  const user = publicUserSchema.safeParse(await response.json())

  if (!user.success) {
    console.log(`Error parsing JSON: ${user.error}`);
    return
  }

  return user.data
}

export const uploadAttachment = async (attachment: FileUpload): Promise<number> => {
  const jwt = getJwt()
  if(!jwt) throw new Error("User is unauthorized: No JWT")

  const response = await fetch("http://localhost:3000/api/upload", {
    method: "POST",
    body: JSON.stringify(attachment),
    headers: {
      "Content-Type": "application/json",
      "authorization": `Bearer ${jwt}`
    }
  })

  if(!response.ok) {
    throw new Error("Error uploading file")
  }

  const result = await response.json()

  // const result = uploadSchema.safeParse(await response.json())
  // if(!result.success) {
  //   throw new Error(`Error parsing file: ${result.error}`)
  // }

  return result.id
}

const mainColors = [
  "bg-main-green",
  "bg-orangey",
  "bg-lilac",
  "bg-main-blue",
  "bg-shock-pink",
];

export const getRandomColor = () => {
  return mainColors[Math.floor(Math.random() * mainColors.length)]
}

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
