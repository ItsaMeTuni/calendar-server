BEGIN TRANSACTION;

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- DESCRIPTION --
-- Creates an api_keys table to store API keys clients will use to
-- make requests.

CREATE TABLE api_keys (
    api_key uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    scopes TEXT[] NOT NULL
);

INSERT INTO schema_changelog (version) VALUES (3);

COMMIT TRANSACTION;