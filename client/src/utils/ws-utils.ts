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
        // message_num: 10
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

export const SocketResponse = {
    FriendRequest: "FriendRequest",
    Message: "Message",
    Generic: "Generic",
    Invite: "Invite",
    Conversation: "Conversation"
}
