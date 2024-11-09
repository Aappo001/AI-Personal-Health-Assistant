import { configureStore } from "@reduxjs/toolkit";
import userReducer from "./userSlice"
import friendsReducer from "./friendsSlice"
import messageReducer from "./messageSlice"

export const store = configureStore({
    reducer: {
        user: userReducer,
        friendsState: friendsReducer,
        messageState: messageReducer
    }
})

export type Rootstate = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch