import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Message } from "../types";

type Conversation = {
    messages: Message[] | undefined,
    title: string | undefined
}

type Conversations = {
    [conversationId: number]: Conversation | undefined
}

type ConversationStore = {
    conversations: Conversations
}

const initialState: ConversationStore = {
    conversations: {}
}

const conversationSlice = createSlice({
    name: "message",
    initialState,
    reducers: {
        pushMessage: (state, action: PayloadAction<{id: number, message: Message}>) => {
            if(action.payload.message.content.trim() === "") return
            const {id, message} = action.payload
            if(!state.conversations[id]) {
                throw new Error(`Conversation Not Defined in PushMessage ${id}: ${state.conversations[id]}`)
            }
            const oldMessages = state.conversations[id].messages ?? []
            //when ai response is finished, server sends a message. Ignore it, as we've already streamed the response
            if(message.userId === undefined && oldMessages[oldMessages.length-1].fromAi) {
                return
            }
            state.conversations[id].messages = [...oldMessages, message]
        },
        pushStreamMessage: (state, action: PayloadAction<{id: number, message: string}>) => {
            const {id, message} = action.payload
            if(!state.conversations[id]) {
                throw new Error("Conversation Not Defined")
            }
            const currentConvo = state.conversations[id].messages ?? []
            const aiMessage: Message = {
                userId: undefined,
                content: message,
                fromAi: true
            }

            if(currentConvo && currentConvo[currentConvo.length-1].fromAi) {
                currentConvo[currentConvo.length-1].content += message
                state.conversations[id].messages = [...currentConvo]
            }
            else {
                state.conversations[id].messages = [...currentConvo, aiMessage]
            }

        },
        initializeConversation: (state, action: PayloadAction<{id: number, title?: string}>) => {
            let {id, title} = action.payload
            if(!title) {
                title = `Conversation ${id}`
            }
            state.conversations = {...state.conversations, [id]: {messages: [], title: title}}
        },
        deleteConversation: (state, action: PayloadAction<number>) => {
            const conversationId = action.payload
            delete state.conversations[conversationId]

        }
    }
})

export const { pushMessage, pushStreamMessage, deleteConversation, initializeConversation } = conversationSlice.actions
export default conversationSlice.reducer