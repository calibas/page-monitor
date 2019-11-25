<?php
// Check for valid site_id and convert to int for DB query
if (!array_key_exists('site_id', $_POST)) {
    http_response_code(400);
    die("site_id required");
}
if (!is_numeric($_POST["site_id"])) {
    http_response_code (400 );
    die("invalid site_id");
}
$site_id = intval($_POST["site_id"]);

// Database setup
$db_host = 'localhost:3306';
$db_user = 'user';
$db_pass = 'password';
$db_connection = mysqli_connect($db_host, $db_user, $db_pass);
mysqli_select_db($db_connection, 'page_monitor');
mysqli_set_charset($db_connection, 'utf8mb4');

$site_query = 'SELECT name, url, crawl_time, res_code, res_time, active  FROM sites WHERE id =' . $site_id;
$site = mysqli_query($db_connection, $site_query, 0 )->fetch_assoc();

if (!$site) {
    http_response_code (400 );
    die("site_id not found");
}

$events_query = 'SELECT id, event_time, difference, event_type  FROM events WHERE site_id =' . $site_id;
$site['events'] = mysqli_query($db_connection, $events_query, 0 )->fetch_all(MYSQLI_ASSOC);

// Prepare output
header('Content-Type: application/json');
header("Access-Control-Allow-Origin: *");
echo json_encode($site);