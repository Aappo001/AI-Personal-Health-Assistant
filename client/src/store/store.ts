import { configureStore } from "@reduxjs/toolkit";
import userReducer from "./userSlice"

export const store = configureStore({
    reducer: {
        user: userReducer
    }
})

export type Rootstate = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch