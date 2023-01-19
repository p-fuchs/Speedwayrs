SELECT
  team_name
FROM
  application.team
WHERE
  team_id = $1;
