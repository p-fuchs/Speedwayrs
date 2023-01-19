SELECT
  player.player_id
FROM
  application.player
WHERE
  player.name = $1 AND player.sname = $2;
