export const wsSendFriendRequest = (ws: WebSocket, friendId: number) => {
    ws.send(
        JSON.stringify({
          type: "SendFriendRequest",
          other_user_id: friendId,
          accept: true,
        })
      );
}

export const SocketResponse = {
    FriendRequest: "FriendRequest",
    Message: "Message",
    Generic: "Generic"
}

// {"type":"FriendRequest","sender_id":1,"receiver_id":2,"created_at":"2024-10-26T04:00:40","status":"Pending"}