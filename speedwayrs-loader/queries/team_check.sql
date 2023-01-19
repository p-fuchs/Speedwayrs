SELECT
  team_id
FROM
  application.team
WHERE
  team_name = $1;
