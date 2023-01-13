CREATE TABLE application.run (
    run_id BIGSERIAL PRIMARY KEY,
    game_id BIGINT REFERENCES application.game NOT NULL,
    place INTEGER REFERENCES application.stadium NOT NULL
);
