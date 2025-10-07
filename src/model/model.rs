use diesel::prelude::*;
use crate::schema::log;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Log {
    pub id: i32,
    pub frequency: i32,
    pub callsign: String,
    pub comment: Option<String>,
    pub bandwidth: i32,
    pub mode: String,
    pub timestamp: std::time::SystemTime,
}


#[derive(Insertable)]
#[diesel(table_name = log)]
pub struct NewLog<'a> {
    pub frequency: i32,
    pub callsign: &'a str,
    pub comment: Option<&'a str>,
    pub bandwidth: i32,
    pub mode: &'a str,
    pub timestamp: std::time::SystemTime,
}
