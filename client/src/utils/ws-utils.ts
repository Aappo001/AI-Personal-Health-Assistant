export const wsSendFriendRequest = (ws: WebSocket, friendId: number) => {
    ws.send(
        JSON.stringify({
          type: "SendFriendRequest",
          other_user_id: friendId,
          accept: true,
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

export const SocketResponse = {
    FriendRequest: "FriendRequest",
    Message: "Message",
    Generic: "Generic",
    Invite: "Invite",
    Conversation: "Conversation",
    Error: "Error"
}
