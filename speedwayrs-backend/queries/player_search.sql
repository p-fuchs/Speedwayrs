SELECT
  player_id,
  name,
  sname
FROM
  application.player
WHERE
  lower(concat(name, sname)) LIKE $1;
