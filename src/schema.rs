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
    }
}

allow_tables_to_appear_in_same_query!(
    analytics,
    blacklist,
    usage_statistics,
    users,
);
