import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { SessionUser } from "../types";

const initialState: SessionUser = {
    id: -1,
    email: "",
    firstName: "",
    lastName: "",
    username: "",
}

const userSlice = createSlice({
    name: "user",
    initialState,
    reducers: {
        updateUser(state, action: PayloadAction<SessionUser>){
            return {...state, ...action.payload}
        }
    }
})

export const {updateUser} = userSlice.actions
export default userSlice.reducer