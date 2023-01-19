SELECT
  run.run_position,
  run.time_integer,
  run.time_decimal
FROM
  application.run
WHERE
  run.game_id = $1;
