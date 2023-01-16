SELECT
  game_id,
  (
    CASE
      WHEN team_1 = $1 THEN n2.team_name
      ELSE n1.team_name
    END
  ) AS opponent,
  (
    CASE
      WHEN team_1 = $1 THEN team_2
      ELSE team_1
    END
  ) AS opponent_id,
  game_date
FROM
  application.game
JOIN application.team n1
  ON n1.team_id = team_1
JOIN application.team n2
  ON n2.team_id = team_2
WHERE
  team_1 = $1
  OR team_2 = $1
ORDER BY
  game_date DESC
OFFSET $2
LIMIT $3;
