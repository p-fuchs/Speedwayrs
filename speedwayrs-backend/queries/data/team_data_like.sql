SELECT
  1 AS liking
FROM
  application.team_like
WHERE
  username = $1 AND team = $2; 
