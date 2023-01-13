SELECT
    team_id,
    team_name
FROM
    team
WHERE
    lower(team_name) LIKE $1;