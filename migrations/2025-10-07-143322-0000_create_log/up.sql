-- Your SQL goes here
CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    frequency FLOAT(16) NOT NULL,
    xcoord  FLOAT(8) NOT NULL DEFAULT 0.0,
    ycoord FLOAT(8) NOT NULL DEFAULT 0.0,
    callsign VARCHAR(50) , 
    comment VARCHAR(50) ,
    mode VARCHAR(20) NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    recording_duration FLOAT(8) NOT NULL
);

