SELECT
  team.team_id,
  team.team_name
FROM
  application.team
WHERE
  team_id = $1 OR team_id = $2;
