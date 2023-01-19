INSERT INTO application.game (team_1, score_1, score_2, team_2, place, game_date)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING game_id;
