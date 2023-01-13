CREATE TABLE application.chat (
    id BIGSERIAL PRIMARY KEY,
    msg TEXT NOT NULL,
    issue_date TIMESTAMPTZ NOT NULL,
    user_id uuid REFERENCES application.user_sessions NOT NULL
);
