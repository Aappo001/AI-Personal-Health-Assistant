import {z} from "zod"

export const registerBodySchema = z.object({
    firstName: z.string().min(1),
    lastName: z.string().min(1),
    username: z.string().min(1),
    email: z.string().min(1),
    password: z.string().min(1)
})