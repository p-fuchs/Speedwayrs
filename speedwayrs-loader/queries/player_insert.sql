INSERT INTO application.player (name, sname)
VALUES ($1, $2)
RETURNING player_id;
