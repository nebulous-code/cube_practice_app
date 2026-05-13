-- Per-case display_rotation: how the case's own pattern is shown across the
-- app (cases browser tile, study session source pattern, case detail top
-- preview). Stored as 0..3 (0°, 90° CW, 180°, 90° CCW) to match the existing
-- result_rotation encoding.
--
-- Override semantics mirror the other user_case_settings columns: NULL on the
-- override row means "no override, fall back to global." Global default is 0
-- so new accounts and existing cases render at the original orientation
-- until the user picks otherwise. No data migration — all rows default to 0.
--
-- Intentionally independent of result_rotation: the post-algorithm result
-- preview keeps its existing behavior and is not composed with this value.

ALTER TABLE cases
    ADD COLUMN display_rotation INT NOT NULL DEFAULT 0
    CHECK (display_rotation BETWEEN 0 AND 3);

ALTER TABLE user_case_settings
    ADD COLUMN display_rotation INT
    CHECK (display_rotation IS NULL OR display_rotation BETWEEN 0 AND 3);
