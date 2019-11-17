table! {
    sites (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        lastcrawl -> Text,
        urls -> Text,
        active -> Bool,
    }
}

table! {
    events (id) {
        id -> Integer,
        site_id -> Integer,
        timestamp -> BigInt,
        difference -> Text,
        event_type -> Text,
    }
}