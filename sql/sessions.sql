CREATE TABLE sessions (
	session_id		VARCHAR(10) NOT NULL UNIQUE,
	user_id				VARCHAR(10) NOT NULL REFERENCES users(user_id),
	expires				DATE NOT NULL
)