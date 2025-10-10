diesel::table! {
    log (id) {
        id -> Int4,
        frequency -> Int4,
        location -> Record<(Float8, Float8)>,
        callsign -> Varchar,
        bandwidth -> Int4,
        mode -> Varchar,
        power -> Nullable<Float8>,
        snr -> Nullable<Float8>,
        comment -> Nullable<Varchar>,
        timestamp -> Timestamp,
    }
}
