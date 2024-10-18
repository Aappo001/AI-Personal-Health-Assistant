import { z } from "zod";

export const registerBodySchema = z.object({
  firstName: z.string().min(1),
  lastName: z.string().min(1),
  username: z.string().min(1),
  email: z.string().min(1),
  password: z.string().min(1),
});

export const sessionUserSchema =  z.object({
    id: z.number(),
    email: z.string(),
    firstName: z.string(),
    lastName: z.string(),
    username: z.string(),
  });

export const implicitLoginSchema = sessionUserSchema

export const loginResponseSchema = z.object({
  message: z.string(),
  user: sessionUserSchema,
});
