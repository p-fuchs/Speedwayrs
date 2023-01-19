SELECT
  player.name,
  player.sname,
  player_score.round,
  player_score.score
FROM
  application.player_score
  LEFT JOIN application.player ON player_score.player_id = player.player_id
WHERE
  game_id = $1;
