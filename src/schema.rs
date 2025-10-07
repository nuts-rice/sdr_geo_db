

diesel::table! {
    log (id) {
        id -> Int4,
        frequency -> Int4,
        callsign -> Varchar,
        comment -> Nullable<Varchar>,
        bandwidth -> Int4,
        mode -> Varchar,
        timestamp -> Timestamp,
    }
}
