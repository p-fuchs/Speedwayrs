SELECT
    users.username, users.email
FROM
    application.users
WHERE
    users.username = $1 OR users.email = $2;
