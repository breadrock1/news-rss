-- Add up migration script here

CREATE TABLE IF NOT EXISTS news(
    id VARCHAR PRIMARY KEY NOT NULL,
    message_url TEXT NOT NULL,
    datetime TIMESTAMP NOT NULL,
    source TEXT,
    photo_path TEXT,
    text TEXT NOT NULL
);
