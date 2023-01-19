WITH team_left_stats AS (
  SELECT
    COALESCE(SUM(CASE WHEN score_1 > score_2 THEN 1 ELSE 0 END), 0) AS wins,
    COALESCE(SUM(CASE WHEN score_1 = score_2 THEN 1 ELSE 0 END), 0) AS ties,
    COALESCE(COUNT(*), 0) AS total
  FROM
    application.game
  WHERE
    team_1 = $1
), team_right_stats AS (
  SELECT
    COALESCE(SUM(CASE WHEN score_1 < score_2 THEN 1 ELSE 0 END), 0) AS wins,
    COALESCE(SUM(CASE WHEN score_1 = score_2 THEN 1 ELSE 0 END), 0) AS ties,
    COALESCE(COUNT(*), 0) AS total
  FROM
    application.game
  WHERE
    team_2 = $1 
)
SELECT
  COALESCE(left_t.wins + right_t.wins, 0) AS wins,
  COALESCE(left_t.ties + right_t.ties, 0) AS ties,
  COALESCE(left_t.total + right_t.total, 0) AS total
FROM
  team_left_stats left_t, team_right_stats right_t;
