ALTER TABLE users ALTER COLUMN hash_tele_num SET NOT NULL;

ALTER TABLE blacklist DROP COLUMN blocker;
ALTER TABLE blacklist DROP COLUMN blocked;

ALTER TABLE blacklist ALTER COLUMN hash_blocker SET NOT NULL;
ALTER TABLE blacklist ALTER COLUMN hash_blocked SET NOT NULL;

ALTER TABLE contacts DROP COLUMN from_tele_num;
ALTER TABLE contacts DROP COLUMN id;
ALTER TABLE contacts ADD PRIMARY KEY (from_id, target_hash_tele_num);

