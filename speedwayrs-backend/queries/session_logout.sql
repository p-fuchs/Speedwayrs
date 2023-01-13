UPDATE application.user_sessions
SET username = NULL
WHERE id = $1;
