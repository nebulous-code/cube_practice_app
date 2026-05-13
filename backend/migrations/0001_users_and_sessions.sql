-- Initial auth schema: users + sessions.
-- See docs/milestones/01_auth_and_accounts.md §3 and docs/Cube_Practice_Design_Doc.md §3.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
    id                         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email                      TEXT UNIQUE NOT NULL,
    pending_email              TEXT,
    display_name               TEXT NOT NULL,
    password_hash              TEXT NOT NULL,
    email_verified             BOOLEAN NOT NULL DEFAULT FALSE,
    verification_code          TEXT,
    verification_code_expires  TIMESTAMPTZ,
    reset_code                 TEXT,
    reset_code_expires         TIMESTAMPTZ,
    streak_count               INT NOT NULL DEFAULT 0,
    last_practice_date         DATE,
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- updated_at trigger keeps the column current on any row update.
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_set_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TABLE sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Fast lookup for sign-out-all and the active-sessions list.
CREATE INDEX sessions_user_id_revoked_idx ON sessions (user_id, revoked);
