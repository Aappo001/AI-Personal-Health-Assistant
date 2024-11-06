CREATE TABLE friend_requests (
	sender_id INTEGER NOT NULL,
	receiver_id INTEGER NOT NULL,
	created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	CHECK (sender_id != receiver_id),
	PRIMARY KEY (sender_id, receiver_id),
	FOREIGN KEY (sender_id) REFERENCES users(id),
        FOREIGN KEY (receiver_id) REFERENCES users(id)
);
