import { RegisterBody, ServerResponse, ErrorResponse } from "../types";

export async function RegisterUser(user: RegisterBody): Promise<ServerResponse | ErrorResponse> {
const response = await fetch("http://localhost:3000/api/register", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(user),
      });

      
      const result: ServerResponse = await response.json();
      if(!response.ok) return {errorMessage: result.message}
      return result
}

