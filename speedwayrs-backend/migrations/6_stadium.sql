CREATE TABLE application.stadium (
    stadium_id SERIAL PRIMARY KEY,
    location_desc TEXT NOT NULL UNIQUE
);
