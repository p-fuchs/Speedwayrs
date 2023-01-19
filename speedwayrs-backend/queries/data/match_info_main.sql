SELECT
  *
FROM
  application.game
  LEFT JOIN application.stadium ON stadium.stadium_id = game.place
WHERE
  game_id = $1;
