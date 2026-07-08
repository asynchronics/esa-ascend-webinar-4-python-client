\c nexosim

CREATE TABLE sine (
    id bigint GENERATED ALWAYS AS IDENTITY,
    ts TIMESTAMP DEFAULT current_timestamp,
    val REAL NOT NULL
);


