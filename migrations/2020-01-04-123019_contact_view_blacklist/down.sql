CREATE OR REPLACE VIEW contact_view AS (
    SELECT contacts.from_id as from_id, c2.name as name, u.firebase_token as firebase_token FROM contacts JOIN users u ON contacts.target_hash_tele_num = u.hash_tele_num JOIN contacts c2 ON contacts.target_hash_tele_num = u.hash_tele_num
);
