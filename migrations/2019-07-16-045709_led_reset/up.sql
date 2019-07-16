SELECT cron.schedule('0 5 * * *', $$UPDATE users SET led = false $$);
