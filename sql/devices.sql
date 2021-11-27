CREATE TABLE devices (
	device_id			VARCHAR(10) NOT NULL UNIQUE,
	user_id				VARCHAR(10) NOT NULL REFERENCES users(user_id),
	device_name		VARCHAR(50) NOT NULL		
)