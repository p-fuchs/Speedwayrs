SELECT
  squad.result
FROM
  application.squad
WHERE
  squad.player_id = $1; 
