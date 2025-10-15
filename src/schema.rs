// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;

    log (id) {
        id -> Int4,
        frequency -> Int4,
        #[max_length = 50]
        callsign -> Varchar,
        xcoord -> Float4,
        ycoord -> Float4,
        bandwidth -> Int4,
        #[max_length = 20]
        mode -> Varchar,
        power -> Nullable<Float8>,
        snr -> Nullable<Float8>,
        #[max_length = 500]
        comment -> Nullable<Varchar>,
        timestamp -> Timestamp,
    }
}
