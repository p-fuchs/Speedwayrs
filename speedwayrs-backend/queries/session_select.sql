SELECT
    user_sessions.expiration,
    user_sessions.username
FROM
    user_sessions
WHERE
    user_sessions.id = $1;