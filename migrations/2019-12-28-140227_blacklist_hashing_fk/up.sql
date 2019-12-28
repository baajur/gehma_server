--ALTER TABLE users ADD CONSTRAINT users_tele_num_unique UNIQUE (tele_num); already there
ALTER TABLE users ADD CONSTRAINT users_tele_num_unique UNIQUE (hash_tele_num);

ALTER TABLE blacklist ADD CONSTRAINT blacklist_hashing_fk_1 FOREIGN KEY (hash_blocker) REFERENCES users (hash_tele_num) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE blacklist ADD CONSTRAINT blacklist_hashing_fk_2 FOREIGN KEY (hash_blocked) REFERENCES users (hash_tele_num) ON UPDATE CASCADE ON DELETE CASCADE;

