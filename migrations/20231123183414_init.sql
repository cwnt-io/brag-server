-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id          SERIAL PRIMARY KEY,
    username    VARCHAR(64) NOT NULL UNIQUE,
    password    VARCHAR(64) NOT NULL,
    deleted_at  TIMESTAMPTZ DEFAULT NULL,
    token       TEXT DEFAULT NULL
);
