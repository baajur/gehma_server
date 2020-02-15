table! {
    users (id) {
        id -> Uuid,
        tele_num -> Varchar,
        led -> Nullable<Bool>,
        created_at -> Timestamp,
    }
}
