WITH liked_players AS (
  SELECT
    player_like.player_id 
  FROM
    application.user_sessions JOIN application.player_like ON player_like.username = user_sessions.username
  WHERE
    user_sessions.id = $1 
)
SELECT
  DISTINCT game.game_id,
  t1.team_name AS team1,
  game.team_1 AS team1_id,
  game.team_2 AS team2_id,
  t2.team_name AS team2,
  game.game_date AS date,
  FORMAT('%s:%s', score_1, score_2) AS score
FROM
  liked_players JOIN application.squad ON liked_players.player_id = squad.player_id
  JOIN application.run ON run.id = squad.run
  JOIN application.game ON game.game_id = run.game_id
  JOIN application.team t1 ON t1.team_id = game.team_1
  JOIN application.team t2 ON t2.team_id = game.team_2
ORDER BY
  date DESC
LIMIT $2;
