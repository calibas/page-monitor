<?php
// respond to preflights
if ($_SERVER['REQUEST_METHOD'] == 'OPTIONS') {
    // return only the headers and not the content
    // only allow CORS if we're doing a GET - i.e. no saving for now.
    if (isset($_SERVER['HTTP_ACCESS_CONTROL_REQUEST_METHOD']) &&
        $_SERVER['HTTP_ACCESS_CONTROL_REQUEST_METHOD'] == 'POST') {
      header('Access-Control-Allow-Origin: *');
      header('Access-Control-Allow-Headers: X-Requested-With, Content-Type');
    }
    die();
}

$json_data = json_decode(file_get_contents('php://input'), true);
// Check for valid site_id and convert to int for DB query
//if (!array_key_exists('site_id', $_POST)) {
if (!array_key_exists('site_id', $json_data)) {
    http_response_code(400);
    die("site_id required");
}
if (!is_numeric($json_data["site_id"])) {
    http_response_code (400 );
    die("invalid site_id");
}
$site_id = intval($json_data["site_id"]);

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
$sevenDaysAgo = strtotime("-7 days");
$events_query = 'SELECT id, event_time, difference, event_type  FROM events WHERE site_id =' . $site_id . ' AND event_time > ' . $sevenDaysAgo . ' ORDER BY id DESC';
$site['events'] = mysqli_query($db_connection, $events_query, 0 )->fetch_all(MYSQLI_ASSOC);

// Prepare output
header('Content-Type: application/json');
header("Access-Control-Allow-Origin: *");
echo json_encode($site);