DELETE FROM application.player_like
WHERE
username = $1
AND player_id = $2;
