ALTER TABLE broadcast ADD COLUMN display_user TEXT NOT NULL REFERENCES users (hash_tele_num) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE broadcast DROP COLUMN display_user_id;
