CREATE TABLE user_statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    height REAL,
    weight REAL,
    sleep_hours REAL,
    exercise_duration REAL,
    food_intake TEXT,
    -- Any extra notes the user wants to add to their statistics
    -- Could include things like how they felt that day, what they did, etc
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TRIGGER user_statistics_update_modified_at
AFTER UPDATE ON user_statistics
BEGIN
    UPDATE user_statistics
    SET modified_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;
