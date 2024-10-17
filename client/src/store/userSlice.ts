import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { UserState } from "../types";

const initialState: UserState = {
    email: "",
    firstName: "",
    lastName: "",
    username: "",
}

const userSlice = createSlice({
    name: "user",
    initialState,
    reducers: {
        updateUser(state, action: PayloadAction<UserState>){
            return {...state, ...action.payload}
        }
    }
})

export const {updateUser} = userSlice.actions
export default userSlice.reducer