INSERT INTO application.team (team_name)
VALUES ($1)
ON CONFLICT DO NOTHING
RETURNING team_id;
