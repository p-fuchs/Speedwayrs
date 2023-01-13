CREATE TABLE IF NOT EXISTS application.users
(
    username VARCHAR(50) PRIMARY KEY,
    password_hash TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE 
);
