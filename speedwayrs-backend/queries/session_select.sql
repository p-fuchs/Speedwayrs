SELECT
    user_sessions.expiration,
    user_sessions.username
FROM
    application.user_sessions
WHERE
    user_sessions.id = $1;
