-- Your SQL goes here
CREATE TABLE IF NOT EXISTS user_<username> (
    id INTEGER PRIMARY KEY,
    message TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);