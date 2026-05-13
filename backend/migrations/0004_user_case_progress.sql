-- M3 schema: per-user-per-case spaced-repetition state.
-- See docs/milestones/03_core_study_loop.md §3 and
-- docs/Cube_Practice_Design_Doc.md §3.
--
-- The set_updated_at() trigger function from migration 0001 is reused here.

CREATE TABLE user_case_progress (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    case_id         UUID NOT NULL REFERENCES cases(id),
    ease_factor     DOUBLE PRECISION NOT NULL DEFAULT 2.5,
    interval_days   INT NOT NULL DEFAULT 1,
    repetitions     INT NOT NULL DEFAULT 0,
    due_date        DATE NOT NULL DEFAULT CURRENT_DATE,
    last_grade      INT,
    last_reviewed   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (user_id, case_id),
    CHECK (interval_days >= 1),
    CHECK (repetitions >= 0),
    CHECK (last_grade IS NULL OR last_grade BETWEEN 0 AND 3)
);

CREATE TRIGGER user_case_progress_set_updated_at
    BEFORE UPDATE ON user_case_progress
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

-- Hot path on GET /study/due (filter by user_id, sort by due_date).
CREATE INDEX user_case_progress_user_due_idx
    ON user_case_progress (user_id, due_date);
