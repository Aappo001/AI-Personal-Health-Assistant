CREATE TABLE ai_models (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL
);

INSERT INTO ai_models (name) VALUES ('mistralai/Mistral-Nemo-Instruct-2407');

CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    message TEXT NOT NULL,
    user_id INTEGER,
    ai_model_id INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    conversation_id INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (ai_model_id) REFERENCES ai_models(id)
);

CREATE TRIGGER update_modified_at AFTER UPDATE ON messages
BEGIN
    UPDATE messages
    SET modified_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER update_last_sent AFTER INSERT ON messages
BEGIN
  UPDATE user_conversations 
  SET last_message_at = CURRENT_TIMESTAMP
  WHERE conversation_id = NEW.conversation_id AND user_id = NEW.user_id;

  UPDATE conversations 
  SET last_message_at = CURRENT_TIMESTAMP
  WHERE id = NEW.conversation_id;
END;

-- References: https://www.sqlite.org/fts5.html and https://www.sqlite.org/fts5.html#external_content_tables
CREATE VIRTUAL TABLE messages_fts USING fts5(conversation_id, message, content='messages', content_rowid='id');

CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages
BEGIN
    INSERT INTO messages_fts(rowid, conversation_id, message) VALUES (NEW.id, NEW.conversation_id,NEW.message);
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages
BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, conversation_id, message) VALUES('delete', OLD.id, OLD.conversation_id, OLD.message);
END;

CREATE TRIGGER messages_fts_update AFTER UPDATE ON messages
BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, conversation_id, message) VALUES('delete', OLD.id, OLD.conversation_id, OLD.message);
    INSERT INTO messages_fts(rowid, conversation_id, message) VALUES (NEW.id, NEW.conversation_id, NEW.message);
END;
