DROP TABLE profile_pictures;

ALTER TABLE users DROP COLUMN profile_picture;
ALTER TABLE users ADD COLUMN profile_picture VARCHAR(256);
