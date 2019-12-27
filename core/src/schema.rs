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
        blocker -> Varchar,
        blocked -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    contacts (id) {
        id -> Int4,
        from_id -> Uuid,
        target_tele_num -> Varchar,
        created_at -> Timestamp,
        name -> Varchar,
        from_tele_num -> Varchar,
        target_hash_tele_num -> Nullable<Bpchar>,
    }
}

table! {
    events (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Varchar,
        opening -> Timestamp,
        closing -> Timestamp,
        country -> Varchar,
        city -> Varchar,
        addr -> Varchar,
        href -> Nullable<Varchar>,
        created_at -> Timestamp,
        changed_at -> Timestamp,
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
        profile_picture -> Varchar,
        firebase_token -> Nullable<Varchar>,
        access_token -> Varchar,
        hash_tele_num -> Nullable<Bpchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    analytics,
    blacklist,
    contacts,
    events,
    usage_statistics,
    users,
);
