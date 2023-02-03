INSERT INTO application.player_like(username, player_id)
VALUES ($1, $2)
ON CONFLICT DO NOTHING;
