CREATE TABLE IF NOT EXISTS emails
(
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  email text NOT NULL
);