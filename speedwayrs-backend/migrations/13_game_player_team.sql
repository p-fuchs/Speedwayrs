CREATE TABLE application.game_team (
  player INTEGER REFERENCES application.player(player_id) NOT NULL,
  team INTEGER REFERENCES application.team(team_id) NOT NULL,
  game INTEGER REFERENCES application.game(game_id) NOT NULL,

  CONSTRAINT player_team_pk PRIMARY KEY (player, team, game)
);
