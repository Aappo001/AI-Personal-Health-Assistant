export const wsSendFriendRequest = (ws: WebSocket, friendId: number, accept: boolean) => {
    ws.send(
        JSON.stringify({
          type: "SendFriendRequest",
          otherUserId: friendId,
          accept: accept,
        })
      );
}

export const wsRequestConversations = (ws: WebSocket) => {
    console.log("Requesting conversations...");
    ws.send(JSON.stringify({
        type: "RequestConversations",
        // last_message_at: null,
        messageNum: 50
    }))
}

export const wsInviteUsersToConvo = (ws: WebSocket, ids: number[]) => {
    console.log("Creating conversation..");
    ws.send(JSON.stringify({
        type: "InviteUsers",
        conversationId: null,
        invitees: ids

    }))
}

export const wsRequestConversation = (ws: WebSocket, id: number) => {
    console.log(`Requesting Conversation ${id} data`);
    ws.send(JSON.stringify({
        type: "RequestConversation",
        conversationId: id
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

export const wsSendMessage = (ws: WebSocket, message: string, conversationId?: number, aiModel?: number) => {
      ws.send(
        JSON.stringify({
          type: "SendMessage",
          message: message.trim() ? message : null,
          conversationId: conversationId,
          aiModelId: aiModel
        })
      );
}

export const wsLeaveConversation = (ws: WebSocket, conversationId: number) => {
    ws.send(JSON.stringify({
        type: "LeaveConversation",
        conversationId: conversationId
    }))
}

export const wsRenameConversation = (ws: WebSocket, conversationId: number, name: string) => {
    ws.send(JSON.stringify({
        type: "RenameConversation",
        conversationId: conversationId,
        name: name
    }))
}

export const SocketResponse = {
    FriendRequest: "FriendRequest",
    Message: "Message",
    Generic: "Generic",
    Invite: "Invite",
    Conversation: "Conversation",
    Error: "Error",
    FriendData: "FriendData",
    StreamData: "StreamData",
    LeaveEvent: "LeaveEvent",
    CanceledGeneration: "CanceledGeneration",
    RenameEvent: "RenameEvent"
}
