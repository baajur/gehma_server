ALTER TABLE users ADD CONSTRAINT id_num_index UNIQUE (tele_num);

CREATE TABLE blacklist (
    id Uuid NOT NULL,
    blocker VARCHAR(100) NOT NULL,
    blocked VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (blocker) REFERENCES users(tele_num) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (blocked) REFERENCES users(tele_num) ON DELETE CASCADE ON UPDATE CASCADE
)

