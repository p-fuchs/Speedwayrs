WITH left_normalized AS(
  SELECT
    game.game_id,
    game.team_1 AS team_2,
    game.score_2 AS score_1,
    game.score_1 AS score_2
  FROM
    application.game
  WHERE
    game.team_2 = $1
),
results AS(
  SELECT
    game.game_id,
    game.team_2,
    game.score_1,
    game.score_2
  FROM
    application.game
  WHERE
    game.team_1 = $1
  UNION ALL 
  SELECT
    *
  FROM
    left_normalized
)SELECT
  results.team_2,
  MAX(team.team_name) AS team_2_name,
  COALESCE(SUM(CASE WHEN results.score_1 > results.score_2 THEN 1 ELSE 0 END), 0) AS wins,
  COALESCE(SUM(CASE WHEN results.score_1 < results.score_2 THEN 1 ELSE 0 END), 0) AS looses,
  COALESCE(COUNT(*), 0) AS games
FROM
  results
JOIN application.team
  ON team.team_id = team_2
GROUP BY
  team_2
HAVING
  COALESCE(SUM(CASE WHEN results.score_1 < results.score_2 THEN 1 ELSE 0 END), 0) > 0
ORDER BY
  looses DESC
LIMIT $2;
