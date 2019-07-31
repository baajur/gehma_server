CREATE TABLE analytics(
    id serial primary key,
    tele_num VARCHAR(100) NOT NULL,
    led BOOLEAN NOT NULL,
    is_autofahrer BOOLEAN NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    FOREIGN KEY (tele_num) REFERENCES users(tele_num) ON DELETE CASCADE ON UPDATE CASCADE
)

