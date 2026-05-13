-- Replace plaintext email in account_deletions with an HMAC-SHA256 hash.
--
-- Existing rows are dropped: the migration runs at launch time when the only
-- rows are launch-day test deletions, and we can't hash them in SQL anyway
-- (the HMAC secret lives in the application env). Re-running this migration
-- is safe; production deployments after this point will only ever see hashed
-- values.

DROP TABLE IF EXISTS account_deletions;

CREATE TABLE account_deletions (
    id          BIGSERIAL PRIMARY KEY,
    email_hash  TEXT        NOT NULL,
    deleted_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_account_deletions_deleted_at ON account_deletions (deleted_at);
CREATE INDEX idx_account_deletions_email_hash ON account_deletions (email_hash);
