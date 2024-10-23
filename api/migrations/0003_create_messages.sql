CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    message TEXT NOT NULL,
    user_id INTEGER,
    ai_model_id INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    conversation_id INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
    FOREIGN KEY (user_id) REFERENCES users(id)
);
