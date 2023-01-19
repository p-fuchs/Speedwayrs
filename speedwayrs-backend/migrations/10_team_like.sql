CREATE TABLE IF NOT EXISTS application.team_like (
  username VARCHAR(50) REFERENCES application.users(username),
  team INTEGER REFERENCES application.team(team_id),
  CONSTRAINT team_like_pk PRIMARY KEY (username, team)
);
