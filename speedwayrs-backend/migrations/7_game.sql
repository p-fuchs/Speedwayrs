CREATE TABLE application.game (
    game_id SERIAL NOT NULL PRIMARY KEY,
    team_1 INTEGER REFERENCES application.team NOT NULL,
    team_2 INTEGER REFERENCES application.team NOT NULL,
    place INTEGER REFERENCES application.stadium NOT NULL,
    game_date TIMESTAMPTZ NOT NULL,

    CONSTRAINT game_teams CHECK (NOT team_1 = team_2)
);
