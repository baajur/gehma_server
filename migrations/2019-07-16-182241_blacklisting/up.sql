ALTER TABLE users ADD CONSTRAINT id_num_index UNIQUE (tele_num);

CREATE TABLE blacklist (
    blocker UUID NOT NULL,
    blocked UUID NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (blocker, blocked),
    FOREIGN KEY (blocker) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (blocked) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
)

