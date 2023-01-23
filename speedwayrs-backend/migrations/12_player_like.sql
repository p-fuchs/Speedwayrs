CREATE TABLE application.player_like (
  username VARCHAR(50) REFERENCES application.users(username) NOT NULL,
  player_id INTEGER REFERENCES application.player(player_id) NOT NULL
);

