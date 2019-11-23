use diesel::prelude::*;
use super::models::*;
use std::fmt;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum EventType {
    HTML_CHANGE,
    LINK_CHANGE,
    WARNING,
    ERROR,
}

// Allow EventType to be used as a string
impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn log_event(conn: &MysqlConnection, e: &str, site_id: i32, etype: EventType) {
    use super::schema::events;
    let new_event = NewEvent {
        site_id: &site_id,
        difference: e,
        event_type: &etype.to_string().to_lowercase(),
    };
    diesel::insert_into(events::table)
        .values(new_event)
        .execute(conn)
        .expect("Error saving new event");
}