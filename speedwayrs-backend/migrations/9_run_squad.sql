CREATE TABLE application.squad (
    entry_id BIGSERIAL PRIMARY KEY,
    run BIGINT REFERENCES application.run NOT NULL,
    player_id INTEGER REFERENCES application.player NOT NULL,
    color VARCHAR(10),

    CONSTRAINT color_enum CHECK (color IN ('Yellow', 'Blue', 'Red', 'White'))
);
