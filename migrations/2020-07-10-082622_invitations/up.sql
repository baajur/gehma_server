CREATE TABLE invitation (
	id SERIAL PRIMARY KEY,
	originator_user_id UUID NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
	edit_text TEXT NOT NULL,
	edit_time TIMESTAMP NOT NULL,
	original_text TEXT NOT NULL,
	original_time TIMESTAMP NOT NULL,
	created_at TIMESTAMP NOT NULL,
	updated_at TIMESTAMP NOT NULL
);

CREATE TABLE invitation_members (
	inv_id INTEGER NOT NULL REFERENCES invitation (id) ON UPDATE CASCADE ON DELETE CASCADE,
	user_id UUID NOT NULL REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
	is_seen BOOL NOt NULL DEFAULT FALSE,
	state INTEGER NOT NULL DEFAULT 2,
	created_at TIMESTAMP NOT NULL,
	updated_at TIMESTAMP NOT NULL,
	PRIMARY KEY(inv_id, user_id)
);