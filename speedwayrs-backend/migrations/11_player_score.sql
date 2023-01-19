CREATE TABLE application.player_score (
  game_id INTEGER REFERENCES application.game NOT NULL,
  player_id INTEGER REFERENCES application.player NOT NULL,
  round INTEGER NOT NULL,
  score VARCHAR(30) NOT NULL,

  CONSTRAINT player_score_pk PRIMARY KEY (game_id, player_id, round)
);
