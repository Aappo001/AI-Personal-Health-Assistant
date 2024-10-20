import { implicitLoginSchema, } from "../schemas";
import { RegisterBody, ServerResponse, ErrorResponse, SessionUser } from "../types";

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

  return result;
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
  const jwt = getJwt()
  if (!jwt) return
  const response = await fetch("http://localhost:3000/api/login", {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      "authorization": `Bearer ${jwt}`
    },
  })
  const data = await response.json()
  if (!response.ok) {
    console.log("Implicit Login Error");
    return
  }
  const parsedData = implicitLoginSchema.safeParse(data)
  if (parsedData.error) {
    console.log(parsedData.error);
    return
  }
  console.log(`Successful Implicit Login: User ${parsedData.data.username}`);

  return parsedData.data

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
  if (!token) return ""
  return token
}

export const saveJwtToLocalStorage = (jwt: string) => {
  localStorage.setItem("token", jwt)
}

export const getJwt = (): string => {
  const jwt = localStorage.getItem("token")
  if (!jwt) return ""
  return jwt
}
