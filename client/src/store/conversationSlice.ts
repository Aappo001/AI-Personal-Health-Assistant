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

// Function to iterate over an array in reverse and return the index that matches the predicate
function findReversed<T>(arr: T[], predicate: (val: T) => boolean | undefined): number | null {
    for (let i = arr.length - 1; i >= 0; i--) {
        if (predicate(arr[i])) {
            return i
        }
    }
    return null
}

const conversationSlice = createSlice({
    name: "message",
    initialState,
    reducers: {
        pushMessage: (state, action: PayloadAction<{ id: number, message: Message }>) => {
            const { id, message } = action.payload
            if (!state.conversations[id]) {
                throw new Error(`Conversation Not Defined in PushMessage ${id}: ${state.conversations[id]}`)
            }
            const oldMessages = state.conversations[id].messages ?? []
            // When ai response is finished, server sends a message, check if we've already streamed the response 
            // in past message already saved in the conversation. If we have, then ignore it
            if (message.userId === undefined && findReversed(oldMessages, (msg) => msg.fromAi && msg.content == message.content)) {
                return
            }
            state.conversations[id].messages = [...oldMessages, message]
        },
        pushStreamMessage: (state, action: PayloadAction<{ id: number, message: string, querierId: number }>) => {
            const { id, message, querierId} = action.payload
            if (!state.conversations[id]) {
                throw new Error("Conversation Not Defined")
            }
            const currentConvo = state.conversations[id].messages ?? []
            const aiMessage: Message = {
                userId: undefined,
                content: message,
                fromAi: true,
                streaming: message != null,
                querierId
            }

            // Iterate over all of the messages in this conversation in reverse in an attempt to map this frame of the 
            // stream to a previously started message.
            //
            // We can identify which stream to append the message to depending on if it is still streaming and if
            // the querierId matches the querierId of the message, as each user is only capable of having one ai 
            // response stream at a time. 
            // If we encounter a message that is not streaming and has the same querierId
            // then we know that this is a new message, so msgIdx will be null and a new message will be appended to 
            // the conversation.
            const msgIdx = findReversed(currentConvo, (msg) => msg.fromAi && msg.streaming && msg.querierId == querierId)

            console.log(`msgIdx: ${msgIdx}`, currentConvo)
            if (msgIdx != null) {
                currentConvo[msgIdx].content += message ? message : ""
                currentConvo[msgIdx].streaming = aiMessage.streaming
                state.conversations[id].messages = [...currentConvo]
            }
            else {
                state.conversations[id].messages = [...currentConvo, aiMessage]
            }

        },
        initializeConversation: (state, action: PayloadAction<{ id: number, title?: string }>) => {
            let { id, title } = action.payload
            if (!title) {
                title = `Conversation ${id}`
            }
            state.conversations = { ...state.conversations, [id]: { messages: [], title: title } }
        },
        deleteConversation: (state, action: PayloadAction<number>) => {
            const conversationId = action.payload
            delete state.conversations[conversationId]

        }
    }
})

export const { pushMessage, pushStreamMessage, deleteConversation, initializeConversation } = conversationSlice.actions
export default conversationSlice.reducer
