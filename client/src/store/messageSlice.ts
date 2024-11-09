import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Message } from "../types";

type Messages = {
    [conversationId: number]: Message[] | undefined
}

type MessageStore = {
    messages: Messages
}

const initialState: MessageStore = {
    messages: {}
}

const messageSlice = createSlice({
    name: "message",
    initialState,
    reducers: {
        pushMessage: (state, action: PayloadAction<{id: number, message: Message}>) => {
            if(action.payload.message.content.trim() === "") return
            const {id, message} = action.payload
            const oldMessages = state.messages[id] ?? []
            state.messages = {...state.messages, [id]: [...oldMessages, message]}
        },
        initializeConversationId: (state, action: PayloadAction<number>) => {
            state.messages = {...state.messages, [action.payload]: undefined}
        }
    }
})

export const { pushMessage, initializeConversationId } = messageSlice.actions
export default messageSlice.reducer