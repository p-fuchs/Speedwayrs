CREATE TABLE application.chat (
  username VARCHAR(50) REFERENCES application.users NOT NULL,
  time TIMESTAMPTZ NOT NULL,
  message TEXT NOT NULL,

  CONSTRAINT chat_pk PRIMARY KEY (username, time, message)
);
