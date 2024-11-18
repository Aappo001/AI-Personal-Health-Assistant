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
    console.log(`Requesting Conversation ${id} data`);
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
        conversation_id: conversationId
    }))
}


    // RenameConversation {
    //     conversation_id: i64,
    //     /// The new name of the conversation
    //     /// If this is None, the frontend should fallback to listing the
    //     /// usernames of the users in the conversation
    //     name: Option<String>,
    // },

export const wsRenameConversation = (ws: WebSocket, conversationId: number, name: string) => {
    ws.send(JSON.stringify({
        type: "RenameConversation",
        conversation_id: conversationId,
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
    CanceledGeneration: "CanceledGeneration"
}
