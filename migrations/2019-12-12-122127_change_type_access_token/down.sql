-- This file should undo anything in `up.sql`
ALTER TABLE users ALTER COLUMN access_token TYPE VARCHAR(256) DEFAULT 'init';
