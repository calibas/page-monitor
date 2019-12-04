#[derive(Queryable)]
pub struct Site {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub last_crawl: String,
    pub crawl_time: i64,
    pub urls: String,
    pub res_code: i32,
    pub res_time: i64,
    pub active: bool,
}

#[derive(Queryable)]
pub struct Event {
    pub id: i32,
    pub site_id: i32,
    pub event_time: i64,
    pub difference: String,
    pub event_type: String,
}

use super::schema::events;
#[derive(Insertable)]
#[table_name="events"]
pub struct NewEvent<'a> {
    pub site_id: &'a i32,
    pub event_time: &'a i64,
    pub difference: &'a str,
    pub event_type: &'a str,
}

use super::schema::sites;
#[derive(Insertable)]
#[table_name="sites"]
pub struct NewSite<'a> {
    pub url: &'a str,
    pub last_crawl: &'a str,
    pub urls: &'a str,
}