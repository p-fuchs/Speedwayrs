SELECT
  game_team.team AS team_id,
  team.team_name,
  COUNT(game_team.game) AS game_count
FROM
  application.game_team JOIN application.team ON game_team.team = team.team_id
WHERE
  game_team.player = $1
GROUP BY
  game_team.team, team.team_name
ORDER BY
  game_count DESC;
