table! {
    sites (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        last_crawl -> Text,
        crawl_time -> BigInt,
        urls -> Text,
        res_code -> Integer,
        res_time -> BigInt,
        active -> Bool,
    }
}

table! {
    events (id) {
        id -> Integer,
        site_id -> Integer,
        timestamp -> Text,
        difference -> Text,
        event_type -> Text,
    }
}