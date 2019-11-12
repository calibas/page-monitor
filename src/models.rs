#[derive(Queryable)]
pub struct Site {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub lastcrawl: String,
    pub active: bool,
}

#[derive(Queryable)]
pub struct Event {
    pub id: i32,
    pub site_id: i32,
    pub timestamp: i64,
    pub difference: String,
    pub event_type: String,
}

use super::schema::events;
#[derive(Insertable)]
#[table_name="events"]
pub struct NewEvent<'a> {
    pub site_id: &'a i32,
    pub difference: &'a str,
}

use super::schema::sites;
#[derive(Insertable)]
#[table_name="sites"]
pub struct NewSite<'a> {
    pub url: &'a str,
}