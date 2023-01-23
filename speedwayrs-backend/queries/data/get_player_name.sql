SELECT
  FORMAT('%s %s', player.name, player.sname) AS name 
FROM
  application.player
WHERE
  player.player_id = $1;
