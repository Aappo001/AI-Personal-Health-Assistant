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
        pushStreamMessage: (state, action: PayloadAction<{id: number, message: string}>) => {
            const {id, message} = action.payload
            const currentConvo = state.messages[id] ?? []
            const aiMessage: Message = {
                userId: undefined,
                content: message,
                fromAi: true
            }

            if(currentConvo && currentConvo[currentConvo.length-1].fromAi) {
                currentConvo[currentConvo.length-1].content += ` ${message}`
                state.messages = {...state.messages, [id]: [...currentConvo]}
            }
            else {
                state.messages = {...state.messages, [id]: [...currentConvo, aiMessage]}
            }

        },
        replaceAiMessage: (state, action: PayloadAction<{id: number, message: string}>) => {
            const {id, message} = action.payload
            const convo = state.messages[id] ?? []
            let index = -1
            for(let i = convo.length-1; i >= 0; i--) {
                if(convo[i].fromAi) {
                    index = i
                    break;
                }
            }
            if(index === -1) {
                throw new Error("Cannot replace nonexistant AI message")
            }
            convo[index].content = message
            state.messages = {...state.messages, [id]: convo}


        },
        initializeConversationId: (state, action: PayloadAction<number>) => {
            state.messages = {...state.messages, [action.payload]: undefined}
        }
    }
})

export const { pushMessage, pushStreamMessage, replaceAiMessage, initializeConversationId } = messageSlice.actions
export default messageSlice.reducer