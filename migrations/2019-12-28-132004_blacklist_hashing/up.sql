ALTER TABLE blacklist ADD COLUMN hash_blocker CHAR(64);
ALTER TABLE blacklist ADD COLUMN hash_blocked CHAR(64);
