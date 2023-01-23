CREATE TABLE application.player (
    player_id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    sname VARCHAR(50) NOT NULL,

    CONSTRAINT player_name UNIQUE (name, sname)
);
