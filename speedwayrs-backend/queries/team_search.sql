SELECT
    team_id,
    team_name
FROM
    application.team
WHERE
    lower(team_name) LIKE $1;
