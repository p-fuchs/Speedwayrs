SELECT
    users.username, users.email
FROM
    users
WHERE
    users.username = $1 OR users.email = $2;