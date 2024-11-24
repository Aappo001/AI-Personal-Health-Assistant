CREATE TABLE ai_models (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	name TEXT NOT NULL
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    message TEXT NOT NULL COLLATE NOCASE,
    stemmed_message TEXT COLLATE NOCASE,
    user_id INTEGER,
    ai_model_id INTEGER,
    file_id INTEGER,
    -- File name is stored in the messages table to allow for
    -- the same file to be uploaded multiple times but 
    -- attached to messages with different names.
    -- Should only be set if file_id is not null
    file_name TEXT CHECK(file_id IS NOT NULL OR file_name IS NULL),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    conversation_id INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (ai_model_id) REFERENCES ai_models(id),
    FOREIGN KEY (file_id) REFERENCES files(id)
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

CREATE VIEW chat_messages AS
SELECT 
	messages.id, 
	messages.message, 
	messages.user_id, 
	messages.ai_model_id, 
	messages.file_name, 
	messages.created_at, 
	messages.modified_at, 
	messages.conversation_id, 
	files.path as file_path
FROM messages
LEFT JOIN files ON messages.file_id = files.id;

-- References: https://www.sqlite.org/fts5.html and https://www.sqlite.org/fts5.html#external_content_tables
CREATE VIRTUAL TABLE messages_fts USING fts5(conversation_id, message, stemmed_message, content='messages', content_rowid='id');

CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages
BEGIN
    INSERT INTO messages_fts(rowid, conversation_id, message, stemmed_message) VALUES (NEW.id, NEW.conversation_id, NEW.message, NEW.stemmed_message);
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages
BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, conversation_id, message, stemmed_message) VALUES('delete', OLD.id, OLD.conversation_id, OLD.message, OLD.stemmed_message);
END;

CREATE TRIGGER messages_fts_update AFTER UPDATE ON messages
BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, conversation_id, message, stemmed_message) VALUES('delete', OLD.id, OLD.conversation_id, OLD.message, OLD.stemmed_message);
    INSERT INTO messages_fts(rowid, conversation_id, message, stemmed_message) VALUES (NEW.id, NEW.conversation_id, NEW.message, NEW.stemmed_message);
END;
