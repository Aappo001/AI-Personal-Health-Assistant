import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Friend } from "../types";

interface FriendsState {
  friends: Friend[];
}

const initialState: FriendsState = {
  friends: [],
};

const friendsSlice = createSlice({
  name: "friends",
  initialState,
  reducers: {
    addFriend: (state, action: PayloadAction<Friend>) => {
      const existingFriend = state.friends.find((friend) => friend.id === action.payload.id);
      if (existingFriend) return;
      state.friends.push(action.payload);
    },
    removeFriend: (state, action: PayloadAction<number>) => {
      state.friends = state.friends.filter((friend) => friend.id !== action.payload);
    },
    updateFriend: (state, action: PayloadAction<Friend>) => {
      const index = state.friends.findIndex((friend) => friend.id === action.payload.id);
      if (index !== -1) {
        state.friends[index] = action.payload;
      }
    },
    upgradeFriendStatus: (state, action: PayloadAction<number>) => {
      const index = state.friends.findIndex((friend) => friend.id === action.payload);
      if (index !== -1) {
        state.friends[index] = {...state.friends[index], status: "Accepted"};
      }
    },
    setFriends: (state, action: PayloadAction<Friend[]>) => {
      state.friends = action.payload;
    },
  },
});

export const { addFriend, removeFriend, updateFriend, setFriends, upgradeFriendStatus } = friendsSlice.actions;
export default friendsSlice.reducer;
