-- This file should undo anything in `up.sql`

ALTER TABLE users ALTER COLUMN led DROP NOT NULL;
