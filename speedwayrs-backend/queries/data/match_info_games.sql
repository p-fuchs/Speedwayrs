SELECT
  run.run_position,
  squad.result,
  player.name,
  player.sname,
  squad.player_id
FROM
  application.run LEFT JOIN application.squad ON squad.run = run.id
  LEFT JOIN application.player ON player.player_id = squad.player_id 
WHERE
  run.game_id = $1;
