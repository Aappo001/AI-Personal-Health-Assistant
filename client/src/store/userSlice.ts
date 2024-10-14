import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { RegisterBody } from "../types";

const initialState: RegisterBody = {
    email: "",
    firstName: "",
    lastName: "",
    username: "",
    password: ""
}

const userSlice = createSlice({
    name: "user",
    initialState,
    reducers: {
        updateUser(state, action: PayloadAction<RegisterBody>){
            return {...state, ...action.payload}
        }
    }
})

export const {updateUser} = userSlice.actions
export default userSlice.reducer