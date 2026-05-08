-- The global nicknames seeded in 0003 ('Tie Fighter', 'Upstairs', etc.) are
-- the original author's personal labels — useful as defaults for him, not
-- meaningful to other users. Strip them from the global cases table so new
-- accounts see no nickname by default and pick their own via per-user
-- overrides (`user_case_settings.nickname`).
--
-- Per-user overrides are unaffected: they live on a different table and the
-- merge logic in `cases/mod.rs` falls through `COALESCE(ucs.nickname, c.nickname)`,
-- which means clearing the global only changes the result when no override
-- is set.
--
-- Idempotent: re-running just sets NULL = NULL.

UPDATE cases
SET nickname = NULL;
