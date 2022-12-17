SELECT
    users.password_hash
FROM
    users
WHERE
    users.username = $1;