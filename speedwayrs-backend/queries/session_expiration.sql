SELECT
    user_sessions.expiration
FROM
    user_sessions
WHERE
    id = $1;