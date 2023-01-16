INSERT INTO application.team_like(username, team)
VALUES ($1, $2)
ON CONFLICT DO NOTHING;
