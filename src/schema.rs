table! {
    blacklist (id) {
        id -> Uuid,
        blocker -> Varchar,
        blocked -> Varchar,
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
        description -> Varchar,
        is_autofahrer -> Bool,
        changed_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    blacklist,
    users,
);
