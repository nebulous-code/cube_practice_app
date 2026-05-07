-- M7: account-deletion audit trail.
--
-- Captures the email + timestamp for every account deletion. Insert-only;
-- written transactionally with the DELETE FROM users so we can never have
-- a deleted user without a corresponding audit row (or vice versa).
--
-- No FK to users — the user row is gone by the time this row is most
-- relevant to query. Email is not unique: register → delete → register-
-- again-with-same-email → delete creates two rows.

CREATE TABLE account_deletions (
    id          BIGSERIAL PRIMARY KEY,
    email       TEXT        NOT NULL,
    deleted_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_account_deletions_deleted_at ON account_deletions (deleted_at);
