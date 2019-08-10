DELETE FROM contacts;
ALTER TABLE contacts DROP CONSTRAINT unique_contact;
ALTER TABLE contacts ADD COLUMN from_tele_num VARCHAR(100) NOT NULL;
--ALTER TABLE contacts DROP COLUMN from_id;
ALTER TABLE contacts add constraint contacts_from_tele_num foreign key(from_tele_num) REFERENCES users(tele_num) ON UPDATE CASCADE ON DELETE CASCADE;
-- ALTER TABLE contacts ADD CONSTRAINT unique_contact UNIQUE(from_id, target_tele_num);
