CREATE TABLE user_statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    height REAL,
    weight REAL,
    sleep_hours REAL,
    exercise_duration REAL,
    food_intake TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_user_statistics_user_id_date ON user_statistics (user_id, date);

CREATE TRIGGER user_statistics_update_modified_at
AFTER UPDATE ON user_statistics
BEGIN
    UPDATE user_statistics
    SET modified_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;
