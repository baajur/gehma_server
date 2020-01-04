CREATE OR REPLACE VIEW contact_view AS (
    SELECT contacts.from_id as from_id, c2.name as name, u.firebase_token as firebase_token FROM contacts JOIN users u ON contacts.target_hash_tele_num = u.hash_tele_num JOIN contacts c2 ON contacts.target_hash_tele_num = u.hash_tele_num JOIN users u2 ON contacts.from_id = u2.id WHERE NOT EXISTS(SELECT * FROM blacklist WHERE (hash_blocker = u2.hash_tele_num and hash_blocked = contacts.target_hash_tele_num) or (hash_blocked = u2.hash_tele_num and hash_blocker = contacts.target_hash_tele_num))
);
