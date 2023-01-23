INSERT INTO application.team (team_name)
VALUES ($1)
RETURNING team_id;
