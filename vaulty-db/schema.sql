SET TIMEZONE = 'America/New_York';

-- Create required tables
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    is_subscribed BOOLEAN NOT NULL, -- Has at least 1 premimum address?
    payment_token TEXT,
    last_update_time TIMESTAMPTZ DEFAULT current_timestamp,
    creation_time TIMESTAMPTZ NOT NULL
);

-- Represents a single email address
CREATE TABLE addresses (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users ON DELETE SET NULL,
    address TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    max_email_size INTEGER NOT NULL, -- Max size of a single email
    quota INTEGER NOT NULL, -- Max number of emails in renewal period
    received INTEGER DEFAULT 0, -- Emails received since last renewal
    last_renewal_time TIMESTAMPTZ NOT NULL,
    last_update_time TIMESTAMPTZ DEFAULT current_timestamp,
    creation_time TIMESTAMPTZ NOT NULL,
    UNIQUE(address, is_active)
);

-- Store logs, one per user
-- Logs are removed if either the associated address or user is deleted
CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users ON DELETE CASCADE,
    address_id INTEGER REFERENCES addresses ON DELETE CASCADE,
    msg TEXT NOT NULL,
    log_level INTEGER NOT NULL,
    is_debug BOOLEAN DEFAULT FALSE, -- If TRUE, this log will not be shown to user
    creation_time TIMESTAMPTZ DEFAULT current_timestamp
);

-- Insert some test data
INSERT INTO users (email, password, is_subscribed, creation_time) VALUES
    ('abc@abc.com', 'test123', FALSE, '2020-02-09 19:38:12-05:00'),
    ('def@abc.com', 'test123', TRUE, '2020-02-09 19:38:12-05:00');

INSERT INTO addresses
    (address, is_active, user_id, max_email_size, quota, last_renewal_time, creation_time) VALUES
    ('info@vaulty.net', TRUE, (SELECT id FROM users WHERE email='abc@abc.com'), 20000000,
     5000, '2020-02-09 19:38:12-05:00','2020-02-09 19:38:12-05:00'),
    ('admin@vaulty.net', TRUE, (SELECT id FROM users WHERE email='def@abc.com'), 20000000,
     5000, '2020-02-09 19:38:12-05:00','2020-02-09 19:38:12-05:00');

INSERT INTO logs (user_id, address_id, msg, log_level, is_debug) VALUES
    (1, 1, 'HELLO THERE 1!', 1, false),
    (1, 2, 'HELLO THERE 2!', 1, false),
    (1, 1, 'HELLO THERE 3!', 1, false);