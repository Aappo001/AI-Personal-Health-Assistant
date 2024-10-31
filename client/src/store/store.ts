import { configureStore } from "@reduxjs/toolkit";
import userReducer from "./userSlice"
import friendsReducer from "./friendsSlice"

export const store = configureStore({
    reducer: {
        user: userReducer,
        friends: friendsReducer
    }
})

export type Rootstate = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch