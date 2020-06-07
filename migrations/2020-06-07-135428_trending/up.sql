CREATE VIEW trending AS SELECT * FROM events WHERE created_at >= NOW()::date ORDER BY (SELECT COUNT(*) FROM votes WHERE events.id = votes.event_id / POW(ABS(EXTRACT(EPOCH FROM NOW() - created_at))/3600, 1.8));

