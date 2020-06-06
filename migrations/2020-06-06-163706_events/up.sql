CREATE SEQUENCE events_id_seq;
ALTER TABLE events DROP CONSTRAINT events_pkey;
ALTER TABLE events DROP COlUMN id;
ALTER TABLE events ADD COlUMN id INTEGER NOT NULL DEFAULT nextval('events_id_seq');
ALTER SEQUENCE events_id_seq OWNED BY events.id;
ALTER TABLE events ADD PRIMARY KEY (id);
ALTER TABLE events DROP COLUMN closing;
ALTER TABLE events ALTER COlUMN href TYPE TEXT;
