CREATE TABLE application.user_sessions (
    id uuid PRIMARY KEY,
    expiration TIMESTAMPTZ NOT NULL,
    username VARCHAR(50) REFERENCES application.users(username)
);
