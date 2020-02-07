-- Your SQL goes here
CREATE OR REPLACE VIEW contact_view AS (
    SELECT contacts.from_id, c2.name, u.firebase_token FROM contacts
    JOIN users u ON contacts.target_hash_tele_num = u.hash_tele_num
    JOIN users u2 ON u2.id = contacts.from_id
    JOIN contacts c2 ON c2.from_id = u.id AND c2.target_hash_tele_num = u2.hash_tele_num
    WHERE NOT EXISTS(SELECT * FROM blacklist WHERE (hash_blocker = u2.hash_tele_num and hash_blocked = contacts.target_hash_tele_num)
);
