extern crate curl;
extern crate structopt;
extern crate diesel;
extern crate diff;

//#[macro_use]
extern crate html5ever;

use std::default::Default;
//use std::iter::repeat;
use std::string::String;
use curl::easy::Easy;
use structopt::StructOpt;
use page_monitor::*;
use page_monitor::models::*;
use diesel::prelude::*;
use html5ever::driver::ParseOpts;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document};
//use diff::lines;

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
    use schema::events;

    let args = Cli::from_args();
    let connection = establish_connection();

    if args.pattern == "add" {
        println!("Hello!");
//        let insert_query = format!("INSERT INTO sites
//            VALUES (NULL,NULL,'{}',NULL)", args.url);
       // pool.prep_exec(insert_query, ()).unwrap();
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
                //parsed_cell.borrow_mut().clear();
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
                println!("{}: {}", site.id, site.url);
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
                    println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id)
                }
            },

            NodeData::Text { ref contents } => {
                previous.push_str(format!("#text: {}\n", escape_default(&contents.borrow())).as_str());
            },

            NodeData::Comment { ref contents } => {
                //previous.push_str(format!("<!-- {} -->\n", escape_default(contents)).as_str());
            },

            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                previous.push_str(format!("<{}", name.local).as_str());
                for attr in attrs.borrow().iter() {
                    if format!("{}",attr.name.local).as_str() != "data-cacheid" {
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