-- Your SQL goes here
CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    frequency INTEGER NOT NULL,
    callsign VARCHAR(50) NOT NULL, 
    bandwidth INTEGER NOT NULL, 
    mode VARCHAR(20) NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  
)
