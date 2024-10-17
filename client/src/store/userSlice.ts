import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { PublicUserState } from "../types";

const initialState: PublicUserState = {
    id: -1,
    firstName: "",
    lastName: "",
    username: "",
}

const userSlice = createSlice({
    name: "user",
    initialState,
    reducers: {
        updateUser(state, action: PayloadAction<PublicUserState>){
            return {...state, ...action.payload}
        }
    }
})

export const {updateUser} = userSlice.actions
export default userSlice.reducer