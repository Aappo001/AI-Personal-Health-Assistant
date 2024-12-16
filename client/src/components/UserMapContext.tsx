import { createContext, SetStateAction, useContext, useState } from "react";
import { PublicUser } from "../types";

export type UserIdMap = {
  [id: number]: PublicUser;
};

const UserIdMapContext = createContext<UserIdMap>({} as UserIdMap);
const UserIdMapDispatchContext = createContext<
  React.Dispatch<SetStateAction<UserIdMap>> | undefined
>(undefined);

export const useUserMapContext = () => {
  const context = useContext(UserIdMapContext);
  if (context === undefined) {
    throw new Error("UserMapContext is undefined");
  }
  return context;
};

export const useUserMapDispatchContext = () => {
  const context = useContext(UserIdMapDispatchContext);
  if (context === undefined) {
    throw new Error("UserMapDispatchContext is undefined");
  }
  return context;
};

export default function UserMapContext({
  children,
  initialState = {},
}: {
  children: React.ReactNode;
  initialState?: UserIdMap;
}) {
  const [userMap, setUserMap] = useState<UserIdMap>(initialState);

  return (
    <>
      <UserIdMapContext.Provider value={userMap}>
        <UserIdMapDispatchContext.Provider value={setUserMap}>
          {children}
        </UserIdMapDispatchContext.Provider>
      </UserIdMapContext.Provider>
    </>
  );
}
