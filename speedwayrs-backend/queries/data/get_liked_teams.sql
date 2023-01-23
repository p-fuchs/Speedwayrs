WITH liked_teams AS (
  SELECT
    team_like.team
  FROM
    application.user_sessions JOIN application.team_like ON team_like.username = user_sessions.username
  WHERE
    user_sessions.id = $1 
)
SELECT
  game.game_id,
  t1.team_name AS team1,
  game.team_1 AS team1_id,
  game.team_2 AS team2_id,
  t2.team_name AS team2,
  FORMAT('%s:%s', score_1, score_2) AS score,
  game.game_date AS date
FROM
  liked_teams JOIN application.game ON (game.team_1 = liked_teams.team OR game.team_2 = liked_teams.team) JOIN application.team t1 ON t1.team_id = game.team_1
  JOIN application.team t2 ON t2.team_id = game.team_2
ORDER BY
  date DESC
LIMIT $2;
