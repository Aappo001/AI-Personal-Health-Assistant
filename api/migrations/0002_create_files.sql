CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    path TEXT NOT NULL,
    mime TEXT,
    -- When the file was first uploaded
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(path, mime)
);

-- Track which user uploaded which files
-- Needs another table to track the many-to-many relationship
-- and allow for multiple users to upload the same file
-- but with different names and not duplicate the file
CREATE TABLE file_uploads (
    file_id INTEGER NOT NULL ,
    user_id INTEGER NOT NULL,
    -- When this user uploaded the file
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (file_id, user_id),
    FOREIGN KEY (file_id) REFERENCES files(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
