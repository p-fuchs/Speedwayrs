SELECT
    users.password_hash
FROM
    application.users
WHERE
    users.username = $1;
