<?php
header('Content-Type: application/json');
header("Access-Control-Allow-Origin: *");

$db_host = 'localhost:3306';
$db_user = 'user';
$db_pass = 'password';

$db_connection = mysqli_connect($db_host, $db_user, $db_pass);

mysqli_select_db($db_connection, 'page_monitor');
mysqli_set_charset($db_connection, 'utf8mb4');

$site_list = array([
    'id' => 0,
    'name' => 'test',
    'url' => 'test',
    'crawl_time' => 14,
    'res_code' => 500,
    'res_time' => '9999',
    'active' => 0
]);
$site_query = 'SELECT * FROM sites ORDER BY id ASC';
$sites = mysqli_query($db_connection, $site_query, 0 );

foreach ($sites as $site) {
    //$site['lastcrawl'] = urlencode($site['lastcrawl']);
    $site_list[] = [
        'id' => (int) $site['id'],
        'name' => $site['name'],
        'url' => $site['url'],
        'crawl_time' => (int) $site['crawl_time'],
        'res_code' => (int) $site['res_code'],
        'res_time' => $site['res_time'],
        'active' => (int) $site['active'],
    ];
}

echo json_encode($site_list);