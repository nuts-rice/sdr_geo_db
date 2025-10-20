// @generated automatically by Diesel CLI.

diesel::table! {
    logs (id) {
        id -> Int4,
        frequency -> Float4,
        xcoord -> Float4,
        ycoord -> Float4,
        #[max_length = 50]
        callsign -> Nullable<Varchar>,
        #[max_length = 50]
        comment -> Nullable<Varchar>,
        #[max_length = 20]
        mode -> Varchar,
        timestamp -> Timestamp,
    }
}
