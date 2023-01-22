SELECT
  *
FROM
  application.chat
ORDER BY
  time DESC
OFFSET $1
LIMIT $2;
