import { z } from "zod";

export const registerBodySchema = z.object({
  firstName: z.string().min(1),
  lastName: z.string().min(1),
  username: z.string().min(1),
  email: z.string().min(1),
  password: z.string().min(1),
});

export const publicUserSchema =  z.object({
    id: z.number(),
    firstName: z.string(),
    lastName: z.string(),
    username: z.string(),
  });

export const implicitLoginSchema = publicUserSchema

export const loginResponseSchema = z.object({
  message: z.string(),
  user: publicUserSchema,
});
