INSERT INTO application.run (run_position, time_integer, time_decimal, game_id)
VALUES ($1, $2, $3, $4)
RETURNING id;
