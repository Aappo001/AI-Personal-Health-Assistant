CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    conversation_id INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
    FOREIGN KEY (user_id) REFERENCES users(id)
);
