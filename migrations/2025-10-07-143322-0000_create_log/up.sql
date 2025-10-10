-- Your SQL goes here
CREATE TABLE log (
    id SERIAL PRIMARY KEY,
    frequency INTEGER NOT NULL,
    location RECORD (
        latitude DOUBLE PRECISION,
        longitude DOUBLE PRECISION
    ) NOT NULL,
    callsign VARCHAR(50) NOT NULL,
    bandwidth INTEGER NOT NULL,
    mode VARCHAR(20) NOT NULL,
    power DOUBLE PRECISION,
    snr DOUBLE PRECISION,
    comment VARCHAR(500),
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
  

