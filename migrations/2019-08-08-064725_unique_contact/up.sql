ALTER TABLE contacts ADD CONSTRAINT unique_contact UNIQUE(from_id, target_tele_num);
