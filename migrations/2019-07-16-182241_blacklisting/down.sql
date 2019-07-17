ALTER TABLE blacklist DROP COLUMN blocker;
ALTER TABLE blacklist DROP COLUMN blocked;
ALTER TABLE users DROP CONSTRAINT id_num_index;

DROP TABLE blacklist;
