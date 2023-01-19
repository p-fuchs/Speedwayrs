INSERT INTO application.stadium (location_desc)
VALUES ($1)
RETURNING stadium_id;
