UPDATE application.user_sessions
SET username = $2, expiration = $3
WHERE id = $1;
