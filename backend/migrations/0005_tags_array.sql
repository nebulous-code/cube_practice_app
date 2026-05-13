-- M4 schema: collapse single-string tier2_tag into a multi-valued tags
-- TEXT[] on cases and user_case_settings. See
-- docs/milestones/04_dashboard_progress_free_study_tags.md §3.
--
-- The override semantics on user_case_settings.tags mirror the other
-- override columns: NULL means "no override, fall back to global." When
-- set, the user array fully replaces the global set. The resolver
-- coerces an empty user array to NULL (no "explicit empty override"
-- state — see §12 of the milestone doc).

-- ─── cases.tags ──────────────────────────────────────────────────────────────
-- NOT NULL DEFAULT '{}' so a case with no tags is an empty array, never
-- NULL — simplifies all downstream code.
ALTER TABLE cases
    ADD COLUMN tags TEXT[] NOT NULL DEFAULT '{}';

UPDATE cases
   SET tags = ARRAY[tier2_tag]
 WHERE tier2_tag IS NOT NULL;

ALTER TABLE cases DROP COLUMN tier2_tag;

-- ─── user_case_settings.tags ─────────────────────────────────────────────────
-- Nullable: NULL = "no override, use global."
ALTER TABLE user_case_settings
    ADD COLUMN tags TEXT[];

UPDATE user_case_settings
   SET tags = ARRAY[tier2_tag]
 WHERE tier2_tag IS NOT NULL;

ALTER TABLE user_case_settings DROP COLUMN tier2_tag;

-- ─── Index ───────────────────────────────────────────────────────────────────
-- GIN supports the eventual "filter cases by any-of tags" query in
-- free-study and the cases browser. Cheap to add now.
CREATE INDEX cases_tags_gin_idx ON cases USING GIN (tags);
