// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "coordinate"))]
    pub struct Coordinate;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Coordinate;

    log (id) {
        id -> Int4,
        frequency -> Int4,
        location -> Coordinate,
        #[max_length = 50]
        callsign -> Varchar,
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
