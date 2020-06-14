table! {
    analytics (id) {
        id -> Int4,
        tele_num -> Varchar,
        led -> Bool,
        description -> Text,
        created_at -> Timestamp,
    }
}

table! {
    blacklist (id) {
        id -> Uuid,
        created_at -> Timestamp,
        hash_blocker -> Bpchar,
        hash_blocked -> Bpchar,
    }
}

table! {
    contacts (from_id, target_hash_tele_num) {
        from_id -> Uuid,
        created_at -> Timestamp,
        name -> Varchar,
        target_hash_tele_num -> Bpchar,
    }
}

table! {
    events (id) {
        name -> Varchar,
        description -> Varchar,
        opening -> Timestamp,
        country -> Varchar,
        city -> Varchar,
        addr -> Varchar,
        href -> Nullable<Text>,
        created_at -> Timestamp,
        changed_at -> Timestamp,
        id -> Int4,
    }
}

table! {
    profile_pictures (id) {
        id -> Int4,
        path -> Text,
    }
}

table! {
    usage_statistics (id) {
        id -> Int4,
        tele_num -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        tele_num -> Varchar,
        led -> Bool,
        created_at -> Timestamp,
        country_code -> Varchar,
        description -> Text,
        changed_at -> Timestamp,
        client_version -> Varchar,
        firebase_token -> Nullable<Varchar>,
        access_token -> Varchar,
        hash_tele_num -> Bpchar,
        xp -> Int4,
        profile_picture -> Int4,
    }
}

table! {
    votes (hash_tele_num, event_id) {
        hash_tele_num -> Varchar,
        event_id -> Int4,
    }
}

joinable!(users -> profile_pictures (profile_picture));
joinable!(votes -> events (event_id));

allow_tables_to_appear_in_same_query!(
    analytics,
    blacklist,
    contacts,
    events,
    profile_pictures,
    usage_statistics,
    users,
    votes,
);
