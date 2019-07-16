-- This file should undo anything in `up.sql`
SELECT cron.unschedule(`SELECT `id` FROM cron.job WHERE command LIKE 'UPDATE users SET led = false');
