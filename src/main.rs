extern crate curl;
extern crate diesel;
extern crate diff;
extern crate html5ever;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate structopt;

use std::default::Default;
use std::string::String;

use page_monitor::*;
use page_monitor::models::*;

use curl::easy::Easy;
use diesel::prelude::*;
use html5ever::driver::ParseOpts;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document};
use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
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
    use schema::events;

    let args = Cli::from_args();
    let connection = establish_connection();

    if args.pattern == "add" {
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
                  `timestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
                  `difference` longtext NOT NULL,
                  `event_type` varchar(255) NOT NULL
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;")
            .execute(&connection)
            .expect("Error creating table events.");
        println!("events table created");
        diesel::sql_query("CREATE TABLE `sites` (
                  `id` int NOT NULL AUTO_INCREMENT PRIMARY KEY,
                  `name` varchar(512) NOT NULL,
                  `url` varchar(512) NOT NULL,
                  `lastcrawl` longtext NOT NULL,
                  `active` tinyint(1) NOT NULL DEFAULT '1'
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;")
            .execute(&connection)
            .expect("Error creating table sites.");
        println!("sites table created");
    }

    if args.pattern == "test" {
        let page_url = args.url;
        let page_string = get_page(&page_url);
        let page_u8 = page_string.as_bytes();
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
        let mut parsed2 = String::new();
        walk(0, &dom.document, &mut parsed2);
        println!("Parsed: {}", parsed2 );
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
                let page_string = get_page(&site.url);
                let page_u8 = page_string.as_bytes();
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
                walk(0, &dom.document, &mut parse2);
                let mut diff_result = String::new();

                for diff in diff::lines(&site.lastcrawl, &parse2) {
                    match diff {
                        diff::Result::Left(_l) => (),//println!("-{}", l),
                        diff::Result::Both(_l, _) => (),//println!(" {}", l),
                        diff::Result::Right(_r) => diff_result.push_str(&format!("+{}\n", _r))//println!("+{}", r)
                    }
                }
                //println!("{}", diff_result);

                if !diff_result.is_empty() {
                    let new_event = NewEvent {
                        site_id: &site.id,
                        difference: &diff_result,
                    };
                    diesel::insert_into(events::table)
                        .values(&new_event)
                        .execute(&connection)
                        .expect("Error saving new post");
                }

                diesel::update(sites.find(site.id))
                    .set(lastcrawl.eq(&parse2))
                    .execute(&connection)
                    .expect("Error connecting to database.");

        }
    }

//    init_tables(pool.clone());

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

fn get_page(url: &str) -> String {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.follow_location(true).unwrap();
    handle.url(url).unwrap();
    handle.perform().unwrap();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    String::from_utf8_lossy(&data).into_owned()
}

#[allow(unused)]
fn walk(indent: usize, handle: &Handle, previous: &mut String)  -> String{
        match handle.data {
            NodeData::Document => {},

            NodeData::Doctype {
                ref name,
                ref public_id,
                ref system_id,
            } => {
                // Doctype is currently disabled in TreeBuilderOpts
                if !name.is_empty() {
                    println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id);
                }
            },

            NodeData::Text { ref contents } => {
                let text = format!("# {}\n", escape_default(&contents.borrow())).as_str().to_owned();
                //let re = Regex::new(r#"(theme_token|cx|view_dom_id|views_dom_id|key)(\\":\\")*[\w|:|-]{8,}"#).unwrap();
                //let result = re.replace_all(&text, "").into_owned();
                //previous.push_str(&result);
                previous.push_str(&text_regex(&text));
            },

            NodeData::Comment { ref contents } => {
                //previous.push_str(format!("<!-- {} -->\n", escape_default(contents)).as_str());
            },

            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let restricted_attrs = vec!["data-cacheid", "id", "class", "value"];
                //in_array("data-cacheid", restricted_attrs);
                previous.push_str(format!("<{}", name.local).as_str());
                for attr in attrs.borrow().iter() {
                    let attr_name = format!("{}",attr.name.local).as_str().to_owned();
                    //if attr_name != "data-cacheid" {
                    if !in_array(&attr_name, &restricted_attrs) {
                        previous.push_str(format!(" {}=\"{}\"", attr.name.local, attr.value).as_str());
                    }
                }
                previous.push_str(format!(">\n").as_str());
            },

            NodeData::ProcessingInstruction { .. } => unreachable!(),
        }

        for child in handle.children.borrow().iter() {
            walk(indent + 4, child, previous);
        }
    previous.to_owned()
}

pub fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}

pub fn in_array(s: &str, arr: &Vec<&str>) -> bool {
    for item in arr.iter() {
        if item == &s {return true}
    }
    false
}

fn text_regex(text: &str) -> String {
    lazy_static! {
        //static ref RE: Regex = Regex::new(r#"(theme_token|cx|view_dom_id|views_dom_id|key)(\\":\\")*[\w|:|-]{8,}"#).unwrap();
        static ref RE: Regex = Regex::new(r#"\[CDATA\[.*?\]\]"#).unwrap();
    }
    RE.replace_all(text, "").into_owned()
}