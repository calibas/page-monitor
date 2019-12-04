extern crate curl;
extern crate diesel;
extern crate diff;
extern crate html5ever;
extern crate pm_rcdom;
extern crate markup5ever;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate structopt;

use std::default::Default;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};

use page_monitor::*;
use page_monitor::models::*;
use page_monitor::event_log::*;
use page_monitor::event_log::EventType::*;

use curl::easy::Easy;
use diesel::prelude::*;
use html5ever::driver::ParseOpts;
use pm_rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::parse_document;
use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /** Valid arguments:
      run - Check each page and save results.
      add - Add new page (req -u).
      create-tables - Create database tables.
      list - List all pages.
      test - Test URL (req -u). */
    pattern: String,

    /// The URL to add
    #[structopt(short = "u", long = "url", default_value = "")]
    url: String,
}

//thread_local!(pub static parsed: RefCell<String> = RefCell::new(String::from("1")));

fn main() {
    use schema::sites::dsl::*;
    //use schema::events::dsl::*;
    use schema::sites;
    //use schema::events;

    let args = Cli::from_args();
    let connection = establish_connection();

    if args.pattern == "add" {
        assert!(&args.url != "", "Requires a URL (use -u)");
        let new_site = NewSite {
            url: &args.url,
        };
        diesel::insert_into(sites::table)
            .values(&new_site)
            .execute(&connection)
            .expect("Error saving new post");
        println!("Added {}", &args.url);
//        let insert_query = format!("INSERT INTO sites
//            VALUES (NULL,NULL,'{}',NULL)", args.url);
        // pool.prep_exec(insert_query, ()).unwrap();
    }

    if args.pattern == "create-tables" {
        diesel::sql_query("CREATE TABLE `events` (
                  `id` int NOT NULL AUTO_INCREMENT PRIMARY KEY,
                  `site_id` int(11) NOT NULL,
                  `event_time` int(11) NOT NULL,
                  `difference` longtext NOT NULL,
                  `event_type` varchar(255) NOT NULL
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;")
            .execute(&connection)
            .expect("Error creating table events.");
        println!("events table created");
        diesel::sql_query("CREATE TABLE `sites` (
                  `id` int NOT NULL AUTO_INCREMENT PRIMARY KEY,
                  `name` varchar(512) NOT NULL DEFAULT '',
                  `url` varchar(512) NOT NULL,
                  `last_crawl` longtext NOT NULL,
                  `crawl_time` int(11) NOT NULL DEFAULT 0,
                  `urls` longtext NOT NULL,
                  `res_code` int(11) NOT NULL DEFAULT 0,
                  `res_time` int(11) NOT NULL DEFAULT 0,
                  `active` tinyint(1) NOT NULL DEFAULT 1
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;")
            .execute(&connection)
            .expect("Error creating table sites.");
        println!("sites table created");
    }

    if args.pattern == "test" {
        assert!(&args.url != "", "Requires a URL (use -u)");
        let page_url = args.url;
        let (page_data, page_code, page_time) = get_page(&connection,&page_url, 0);
        let page_u8 = page_data.as_bytes();
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut page_u8.clone())
            .unwrap();
        let mut walked = String::new();
        let mut new_urls = String::new();
        walk(0, &dom.document, &mut walked, &mut new_urls);
        println!("Parsed: {}", walked);
        println!("{} {} {}", page_code, page_time, page_url);
    }

    if args.pattern == "run" {
        let results = sites
            .filter(active.eq(true))
            //.limit(5)
            .load::<Site>(&connection)
            .expect("Error loading sites.");
        println!("Fetching {} pages", results.len());
        for site in results {
            println!("{}: {}", site.id, site.url);
            let (page_data, page_code, page_time) = get_page(&connection, &site.url, site.id);
            let page_u8 = page_data.as_bytes();
            let opts = ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: true,
                    ..Default::default()
                },
                ..Default::default()
            };
            let dom = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut page_u8.clone())
                .unwrap();
            let mut parse2 = String::new();
            let mut new_urls = String::new();
            walk(0, &dom.document, &mut parse2, &mut new_urls);

            // Check for HTML differences
            let mut diff_result = String::new();
            for diff in diff::lines(&site.last_crawl, &parse2) {
                match diff {
                    diff::Result::Left(_l) => (),//println!("-{}", l),
                    diff::Result::Both(_l, _) => (),//println!(" {}", l),
                    diff::Result::Right(_r) => diff_result.push_str(&format!("+{}\n", _r))//println!("+{}", r)
                }
            }

            if !diff_result.is_empty() {
                log_event(&connection, &diff_result,site.id as i32, HTML_CHANGE);
//                let new_event = NewEvent {
//                    site_id: &site.id,
//                    difference: &diff_result,
//                    event_type: "html_change",
//                };
//                diesel::insert_into(events::table)
//                    .values(&new_event)
//                    .execute(&connection)
//                    .expect("Error saving new event");
            }

            //Check for link differences
            let mut diff_link = String::new();
            for diff2 in diff::lines(&site.urls, &new_urls) {
                match diff2 {
                    diff::Result::Left(_l) => (diff_link.push_str(&format!("-{}\n", _l))),//println!("-{}", l),
                    diff::Result::Both(_l, _) => (),//println!(" {}", l),
                    diff::Result::Right(_r) => diff_link.push_str(&format!("+{}\n", _r))//println!("+{}", r)
                }
            }

            if !diff_link.is_empty() {
                log_event(&connection, &diff_link,site.id as i32, LINK_CHANGE);
//                let new_event = NewEvent {
//                    site_id: &site.id,
//                    difference: &diff_link,
//                    event_type: "link_change",
//                };
//                diesel::insert_into(events::table)
//                    .values(&new_event)
//                    .execute(&connection)
//                    .expect("Error saving new event");
            }

            let fetch_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel detected.").as_secs() as i64;

            diesel::update(sites.find(site.id))
                .set((
                    last_crawl.eq(&parse2),
                    crawl_time.eq(fetch_time),
                    urls.eq(new_urls),
                    res_code.eq(page_code),
                    res_time.eq(page_time)
                ))
                .execute(&connection)
                .expect("Error connecting to database.");
        }
    }

    if args.pattern == "list" {
        let results = sites
            //.filter(active.eq(true))
            //.limit(5)
            .load::<Site>(&connection)
            .expect("Error loading sites.");
        println!("Displaying {} posts", results.len());
        for site in results {
            println!("{}: {}  active: {}", site.id, site.url, site.active);
        }
    }
}

fn get_page(conn: &MysqlConnection, url: &str, site_id: i32) -> (String, i32, i64 ) {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    let mut res_code :u32 = 0;
    let mut res_time :u128 = 0;
    handle.follow_location(true).unwrap();
    handle.url(url).unwrap();
    match handle.perform() {
        Ok(_f) =>
            {
                res_code = handle.response_code().unwrap();
                res_time = handle.total_time().unwrap().as_millis();
                println!("{} {}", res_code, res_time);
                if res_code != 200 {
                    log_event(conn, &format!("Bad response ({})",res_code),site_id, ERROR);
                }
                if res_time > 2000 {
                    log_event(conn, &format!("Slow response time ({})",res_time),site_id, WARNING);
                }
                let mut transfer = handle.transfer();
                transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                }).unwrap();
                transfer.perform().unwrap();
            },
        Err(e) => {
            println!("Connection error \n{:?}", e);   //handled error
            //log_error_event(&format!("Connection error {}",e),site_id);
            log_event(conn, &format!("Connection error ({})",e), site_id, ERROR );
        }
    }
    (String::from_utf8_lossy(&data).into_owned(), res_code as i32, res_time as i64 )
}

#[allow(unused)]
fn walk(indent: usize, handle: &Handle, previous: &mut String, urls: &mut String) -> String {
    match handle.data {
        NodeData::Document => {}

        NodeData::Doctype {
            ref name,
            ref public_id,
            ref system_id,
        } => {
            // Doctype is currently disabled in TreeBuilderOpts
            if !name.is_empty() {
                println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id);
            }
        }

        NodeData::Text { ref contents } => {
            let text = format!("# {}\n", escape_default(&contents.borrow())).as_str().to_owned();
            //let re = Regex::new(r#"(theme_token|cx|view_dom_id|views_dom_id|key)(\\":\\")*[\w|:|-]{8,}"#).unwrap();
            previous.push_str(&text_regex(&text));
        }

        NodeData::Comment { ref contents } => {
            //previous.push_str(format!("<!-- {} -->\n", escape_default(contents)).as_str());
        }

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let restricted_attrs = vec!["data-cacheid", "id", "class", "value"];
            previous.push_str(format!("<{}", name.local).as_str());
            for attr in attrs.borrow().iter() {
                let attr_name = format!("{}", attr.name.local).as_str().to_owned();
                if !in_array(&attr_name, &restricted_attrs) {
                    previous.push_str(format!(" {}=\"{}\"", attr.name.local, attr.value).as_str());
                }
                if attr_name == "href" && format!("{}", name.local).as_str() == "a" {
                    //println!("Link found {}", attr.value);
                    urls.push_str(format!("{}\n", attr.value).as_str());
                }
            }
            previous.push_str(format!(">\n").as_str());
        }

        NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    for child in handle.children.borrow().iter() {
        walk(indent + 4, child, previous, urls);
    }
    previous.to_owned()
}

pub fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}

pub fn in_array(s: &str, arr: &Vec<&str>) -> bool {
    for item in arr.iter() {
        if item == &s { return true; }
    }
    false
}

fn text_regex(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"\[CDATA\[.*?\]\]"#).unwrap();
    }
    RE.replace_all(text, "").into_owned()
}

//fn log_error_event(e: &str, site_id: i32) {
//    use schema::events;
//    let new_event = NewEvent {
//        site_id: &site_id,
//        difference: e,
//        event_type: "error",
//    };
//    let connection = establish_connection();
//    diesel::insert_into(events::table)
//        .values(new_event)
//        .execute(&connection)
//        .expect("Error saving new event");
//}