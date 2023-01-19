CREATE TABLE application.squad (
    id BIGSERIAL PRIMARY KEY,
    run BIGINT REFERENCES application.run NOT NULL,
    player_id INTEGER REFERENCES application.player NOT NULL,
    result VARCHAR(30) NOT NULL,
    color VARCHAR(10),

    CONSTRAINT color_enum CHECK (color IN ('Yellow', 'Blue', 'Red', 'White'))
);
