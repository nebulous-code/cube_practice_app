-- M5: track whether a user has seen the post-verification onboarding stub.
-- Backend column instead of localStorage so the flag survives device changes
-- and storage clears. See docs/milestones/05_polish_and_static_pages.md §3.
--
-- Pre-launch we accept that any existing rows backfill to FALSE — they would
-- re-see onboarding on next verification landing, which is fine since there
-- are no production users yet. Post-launch, a similar migration would want a
-- created_at-based backfill condition.

ALTER TABLE users
    ADD COLUMN has_seen_onboarding BOOLEAN NOT NULL DEFAULT FALSE;
