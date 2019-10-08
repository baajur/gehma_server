table! {
    analytics (id) {
        id -> Int4,
        tele_num -> Varchar,
        led -> Bool,
        is_autofahrer -> Bool,
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
        is_autofahrer -> Bool,
        changed_at -> Timestamp,
        client_version -> Varchar,
        firebase_token -> Nullable<Varchar>,
        profile_picture -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    analytics,
    blacklist,
    contacts,
    usage_statistics,
    users,
);
