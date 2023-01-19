SELECT
  stadium_id
FROM
  application.stadium
WHERE
  location_desc = $1; 
