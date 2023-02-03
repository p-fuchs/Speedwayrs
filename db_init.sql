CREATE SCHEMA application;
CREATE USER srs_backend WITH PASSWORD 'srs-dev';
GRANT ALL ON SCHEMA application TO srs_backend;
