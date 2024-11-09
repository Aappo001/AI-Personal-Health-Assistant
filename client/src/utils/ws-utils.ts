export const wsSendFriendRequest = (ws: WebSocket, friendId: number, accept: boolean) => {
    ws.send(
        JSON.stringify({
          type: "SendFriendRequest",
          other_user_id: friendId,
          accept: accept,
        })
      );
}

export const wsRequestConversations = (ws: WebSocket) => {
    console.log("Requesting conversations...");
    ws.send(JSON.stringify({
        type: "RequestConversations",
        // last_message_at: null,
        message_num: 50
    }))
}

export const wsInviteUsersToConvo = (ws: WebSocket, ids: number[]) => {
    console.log("Creating conversation..");
    ws.send(JSON.stringify({
        type: "InviteUsers",
        conversation_id: null,
        invitees: ids

    }))
}

export const wsRequestConversation = (ws: WebSocket, id: number) => {
    console.log("Requesting singular conversation");
    ws.send(JSON.stringify({
        type: "RequestConversation",
        conversation_id: id
    }))
}

export const wsRequestMessages = (ws: WebSocket, id: number) => {
    console.log("Requesting messages");
    ws.send(JSON.stringify({
        type: "RequestMessages",
        conversationId: id
    }))
}

export const wsRequestFriends = (ws: WebSocket) => {
    console.log("Requesting friends");
    ws.send(JSON.stringify({
        type: "RequestFriends"
    }))
}

export const wsRequestFriendRequests = (ws: WebSocket) => {
    ws.send(JSON.stringify({
        type: "RequestFriendRequests"
    }))
}

export const SocketResponse = {
    FriendRequest: "FriendRequest",
    Message: "Message",
    Generic: "Generic",
    Invite: "Invite",
    Conversation: "Conversation",
    Error: "Error",
    FriendData: "FriendData"
}
