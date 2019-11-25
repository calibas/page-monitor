<?php
echo "<html lang='en'>";
echo "<head>><title>Page monitor</title></head>";
echo "<link rel='stylesheet' href='style.css'>";
echo "</head>";
echo "<body>";

$db_host = 'localhost:3306';
$db_user = 'user';
$db_pass = 'password';

$db_connection = mysqli_connect($db_host, $db_user, $db_pass);

mysqli_select_db($db_connection, 'page_monitor');
mysqli_set_charset($db_connection, 'utf8mb4');

$site_list = array("test");
$site_query = 'SELECT * FROM sites ORDER BY id ASC';
$sites = mysqli_query($db_connection, $site_query, 0 );

foreach ($sites as $site) {
    $site_list[$site['id']] = $site['url'];
}

$event_query = 'SELECT * FROM events ORDER BY id DESC LIMIT 50';
$events = mysqli_query($db_connection, $event_query, 0 );

echo "<table class='values'>";
echo "<tr><th>id</th><th>info</th><th>diff</th></tr>";
foreach ($events as $event) {
    $encoded = htmlspecialchars($event['difference']);
    echo "<tr class='event-type-{$event['event_type']}'><td>{$event['id']}</td>".
        "<td class='{$event['site_id']}'><div class='event-site'>{$site_list[$event['site_id']]} ({$event['site_id']})</div>".
        "<div class='event-time'>{$event['event_time']}</div>".
        "<div class='event-type'>{$event['event_type']}</div></td>".
        "<td class='event-diff-td'><div class='event-diff'>{$encoded}</div></td></tr>";
}
echo "</table>";

mysqli_close($db_connection);
echo "</body></html>";