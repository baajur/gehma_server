-- This file should undo anything in `up.sql`
ALTER TABLE contacts ADD COLUMN target_tele_num VARCHAR(100) NOT NULL FOREIGN KEY (target_tele_num) REFERENCES users(tele_num) ON DELETE CASCADE ON UPDATE CASCADE;

