CREATE TABLE profile_pictures (
	id SERIAL PRIMARY KEY,
	path TEXT NOT NULL
);

ALTER TABLE users DROP COLUMN profile_picture;
ALTER TABLE users ADD COLUMN profile_picture INTEGER REFERENCES profile_pictures (id) ON DELETE SET NULL;
