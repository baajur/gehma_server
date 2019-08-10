DELETE FROM contacts;
--ALTER TABLE contacts ADD COLUMN from_id UUID NOT NULL;
ALTER TABLE contacts DROP COLUMN from_tele_num;
--ALTER TABLE contacts ADD CONSTRAINT unique_contact UNIQUE(from_id, target_tele_num);
--ALTER TABLE contacts add constraint contacts_from_id foreign key(from_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE;
