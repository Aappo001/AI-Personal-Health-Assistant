CREATE TABLE user_settings (
    user_id INTEGER NOT NULL PRIMARY KEY,
    ai_model_id INTEGER,
    ai_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    theme TEXT NOT NULL DEFAULT 'dark',
    modified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
    FOREIGN KEY (ai_model_id) REFERENCES ai_models(id)
);

CREATE TRIGGER user_settings_update_modified_at
AFTER UPDATE ON user_settings
BEGIN
    UPDATE user_settings
    SET modified_at = CURRENT_TIMESTAMP
    WHERE user_id = NEW.user_id;
END;
