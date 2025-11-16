-- Add migration script here
CREATE TABLE IF NOT EXISTS ids (discord_id INTEGER PRIMARY KEY, tempus_id INTEGER NOT NULL UNIQUE)
