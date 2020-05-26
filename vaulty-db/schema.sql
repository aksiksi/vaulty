-- Writes some initial test data to the DB
-- The schema itself is maintained by the Django ORM under vaulty-web

-- Cleanup all test data
DELETE FROM vaulty_aliases
WHERE alias = 'postmaster@vaulty.net' OR alias = 'support@vaulty.net';

DELETE FROM vaulty_logs
WHERE email_id = '00000000-0000-0000-0000-000000000000';

DELETE FROM vaulty_emails
WHERE id = '00000000-0000-0000-0000-000000000000';

DELETE FROM vaulty_addresses
WHERE address = 'test1@vaulty.net' OR address = 'test2@vaulty.net';

DELETE FROM vaulty_users
WHERE email_address = 'abc@abc.com' OR email_address = 'def@abc.com';

-- Insert new test data
INSERT INTO vaulty_users (email_address, password, is_subscribed, last_update_time, creation_time) VALUES
    ('abc@abc.com', 'test123', FALSE, '2020-02-09 19:38:12-05:00', '2020-02-09 19:38:12-05:00'),
    ('def@abc.com', 'test123', TRUE, '2020-02-09 19:38:12-05:00', '2020-02-09 19:38:12-05:00');

INSERT INTO vaulty_addresses
    (address, is_active, user_id, email_quota, num_received, max_email_size, storage_quota, storage_used, last_renewal_time, last_update_time, creation_time, storage_backend, storage_token, storage_path, whitelist, is_whitelist_enabled) VALUES
    ('test1@vaulty.net', TRUE, (SELECT id FROM vaulty_users WHERE email_address='abc@abc.com'), 1000, 0, 20000000,
     20000000000, 0, '2020-02-09 19:38:12-05:00', '2020-02-09 19:38:12-05:00', '2020-02-09 19:38:12-05:00', 'dropbox', '{{ vaulty_dropbox_token }}', '/vaulty', '{"cyph0nik@gmail.com"}', true),
    ('test2@vaulty.net', TRUE, (SELECT id FROM vaulty_users WHERE email_address='def@abc.com'), 100, 0, 20000000, 40000000, 0, '2020-02-09 19:38:12-05:00', '2020-02-09 19:38:12-05:00', '2020-02-09 19:38:12-05:00', 'gdrive', 'testabc', '/vaulty/', '{}', false);

INSERT INTO vaulty_emails (user_id, address_id, id, num_attachments,
                      total_size, status, error_msg, creation_time, last_update_time) VALUES
    ((SELECT id FROM vaulty_users WHERE email_address='abc@abc.com'), (SELECT id FROM vaulty_addresses WHERE address='test1@vaulty.net'), '00000000-0000-0000-0000-000000000000', 10, 10000, true,
     'NO ERROR', '2020-02-09 19:38:12-05:00', '2020-02-09 19:38:12-05:00');

INSERT INTO vaulty_logs (email_id, msg, log_level, creation_time) VALUES
    ('00000000-0000-0000-0000-000000000000', 'HELLO THERE 1!', 1, '2020-02-09 19:38:12-05:00'),
    ('00000000-0000-0000-0000-000000000000', 'HELLO THERE 2!', 1, '2020-02-09 19:38:12-05:00'),
    ('00000000-0000-0000-0000-000000000000', 'HELLO THERE 3!', 1, '2020-02-09 19:38:12-05:00');

INSERT INTO vaulty_aliases (is_active, alias, dest) VALUES
    (true, 'postmaster@vaulty.net', 'vmail@localhost'),
    (true, 'support@vaulty.net', 'vmail@localhost');
