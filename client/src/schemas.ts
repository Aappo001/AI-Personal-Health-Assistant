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

export const appErrorSchema = z.object({
  message: z.string(),
  type: z.string()
})

export const userChatMessageSchema = z.object({
  type: z.literal("Message"),
  id: z.number(),
  conversationId: z.number(),
  message: z.string(),
  createdAt: z.string(),
  modifiedAt: z.string()
}).or(appErrorSchema)

export const conversationSchema = z.object({
  id: z.number(),
  title: z.optional(z.string()),
  createdAt: z.string(),
  lastMessageAt: z.string()
})

export const inviteSchema = z.object({
  conversationId: z.number(),
  inviter: z.number(),
  invitedAt: z.string()
})

export const friendRequestSchema = z.object({
  senderId: z.number(),
  receiverId: z.string(),
  createdAt: z.string(),
  status: z.string()
})

export const readEventSchema = z.object({
  conversationId: z.number(),
  userId: z.number(),
  timestamp: z.string()
})
