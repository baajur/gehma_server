CREATE TABLE usage_statistics (
    id SERIAL PRIMARY KEY,
    tele_num VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL
)
