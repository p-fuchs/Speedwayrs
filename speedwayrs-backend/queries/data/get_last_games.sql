SELECT
  game.game_id,
  t1.team_name AS team1,
  game.team_1 AS team1_id,
  game.team_2 AS team2_id,
  t2.team_name AS team2,
  FORMAT('%s:%s', score_1, score_2) AS score,
  game.game_date AS date
FROM
  application.game JOIN application.team t1 ON t1.team_id = game.team_1
  JOIN application.team t2 ON t2.team_id = game.team_2
ORDER BY
  date DESC
LIMIT $1
OFFSET $2;
