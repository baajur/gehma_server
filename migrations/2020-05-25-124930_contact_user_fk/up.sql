ALTER TABLE contacts 
ADD CONSTRAINT fk_contacts_target_hash FOREIGN KEY (target_hash_tele_num) REFERENCES users (hash_tele_num) ON UPDATE CASCADE ON DELETE CASCADE;
