SET TIMEZONE = 'America/New_York';

DROP TABLE IF EXISTS logs, addresses, emails, users;

CREATE TYPE storage_backend AS ENUM ('dropbox', 'gdrive', 's3');

-- Single user
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    is_subscribed BOOLEAN NOT NULL,
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
    storage_backend storage_backend NOT NULL,
    storage_token TEXT NOT NULL, -- Token for whichever provider is used
    storage_path TEXT NOT NULL, -- Path to store data (in valid backend format)
    last_update_time TIMESTAMPTZ DEFAULT current_timestamp,
    creation_time TIMESTAMPTZ NOT NULL,
    UNIQUE(address, is_active)
);

-- Tracks data about each email received by server
CREATE TABLE emails (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users ON DELETE CASCADE,
    address_id INTEGER REFERENCES addresses ON DELETE CASCADE,
    email_id UUID NOT NULL,
    message_id TEXT, -- Standard MIME Message-ID
    num_attachments INTEGER NOT NULL,
    total_size INTEGER NOT NULL,
    status BOOLEAN DEFAULT true, -- Email processed successfully by default
    error_msg TEXT,
    last_update_time TIMESTAMPTZ DEFAULT current_timestamp,
    creation_time TIMESTAMPTZ NOT NULL,
    UNIQUE(email_id)
);

-- Logs used for debugging issues
CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    -- NOTE: email_id can be NULL (e.g., log w/o inserting an email)
    email_id UUID REFERENCES emails(email_id) ON DELETE CASCADE,
    msg TEXT NOT NULL,
    log_level INTEGER NOT NULL,
    creation_time TIMESTAMPTZ DEFAULT current_timestamp
);

-- Insert some test data
INSERT INTO users (email, password, is_subscribed, creation_time) VALUES
    ('abc@abc.com', 'test123', FALSE, '2020-02-09 19:38:12-05:00'),
    ('def@abc.com', 'test123', TRUE, '2020-02-09 19:38:12-05:00');

INSERT INTO addresses
    (address, is_active, user_id, max_email_size, quota, last_renewal_time, creation_time, storage_backend, storage_token, storage_path) VALUES
    ('info@vaulty.net', TRUE, (SELECT id FROM users WHERE email='abc@abc.com'), 20000000,
     5000, '2020-02-09 19:38:12-05:00','2020-02-09 19:38:12-05:00', 'dropbox', '{{ vaulty_dropbox_token }}', '/vaulty'),
    ('admin@vaulty.net', TRUE, (SELECT id FROM users WHERE email='def@abc.com'), 20000000, 5000, '2020-02-09 19:38:12-05:00','2020-02-09 19:38:12-05:00', 'gdrive', 'testabc', '/vaulty/');

INSERT INTO emails (user_id, address_id, email_id, num_attachments,
                      total_size, status, error_msg, creation_time) VALUES
    (1, 1, '00000000-0000-0000-0000-000000000000', 10, 10000, true,
     'NO ERROR', '2020-02-09 19:38:12-05:00');

INSERT INTO logs (email_id, msg, log_level) VALUES
    ('00000000-0000-0000-0000-000000000000', 'HELLO THERE 1!', 1),
    ('00000000-0000-0000-0000-000000000000', 'HELLO THERE 2!', 1),
    ('00000000-0000-0000-0000-000000000000', 'HELLO THERE 3!', 1);
