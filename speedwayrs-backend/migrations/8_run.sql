CREATE TABLE application.run (
    id BIGSERIAL PRIMARY KEY,
    run_position INTEGER NOT NULL,
    time_integer INTEGER,
    time_decimal INTEGER,
    game_id INTEGER REFERENCES application.game NOT NULL,

    CONSTRAINT time_consistency CHECK ((time_integer IS NULL AND time_decimal IS NULL) OR (NOT(time_integer IS NULL OR time_decimal IS NULL)))
);
