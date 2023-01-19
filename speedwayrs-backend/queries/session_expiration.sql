SELECT
    user_sessions.expiration
FROM
    application.user_sessions
WHERE
    id = $1;
