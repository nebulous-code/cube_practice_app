-- M2 schema: puzzle types, solve stages, canonical cases, and per-user
-- case overrides. See docs/milestones/02_case_data_and_browser.md §3 and
-- docs/Cube_Practice_Design_Doc.md §3.
--
-- The set_updated_at() trigger function from 0001 is reused here for
-- user_case_settings so we don't redefine it.

CREATE TABLE puzzle_types (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT UNIQUE NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE solve_stages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    puzzle_type_id  UUID NOT NULL REFERENCES puzzle_types(id),
    name            TEXT NOT NULL,
    description     TEXT,
    display_order   INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (puzzle_type_id, name)
);

-- result_case_id is nullable to break the chicken/egg during the seed —
-- all 57 rows are inserted first, then result_case_id is backfilled from
-- the (case_number, result_rotation) mapping. After seed, every row has
-- a non-null result_case_id; we don't enforce it at the column level
-- because there's no clean way to add NOT NULL mid-seed without a
-- deferrable FK dance.
CREATE TABLE cases (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    solve_stage_id   UUID NOT NULL REFERENCES solve_stages(id),
    case_number      INT NOT NULL,
    nickname         TEXT,
    algorithm        TEXT NOT NULL,
    result_case_id   UUID REFERENCES cases(id),
    result_rotation  INT NOT NULL DEFAULT 0,
    diagram_data     JSONB NOT NULL,
    tier1_tag        TEXT NOT NULL,
    tier2_tag        TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (solve_stage_id, case_number),
    CHECK (result_rotation BETWEEN 0 AND 3),
    CHECK (tier1_tag IN ('+', '-', 'L', '*'))
);

-- Per-user override. NULL in any column = fall back to the global cases row.
CREATE TABLE user_case_settings (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    case_id          UUID NOT NULL REFERENCES cases(id),
    nickname         TEXT,
    algorithm        TEXT,
    result_case_id   UUID REFERENCES cases(id),
    result_rotation  INT,
    tier2_tag        TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (user_id, case_id),
    CHECK (result_rotation IS NULL OR result_rotation BETWEEN 0 AND 3)
);

CREATE TRIGGER user_case_settings_set_updated_at
    BEFORE UPDATE ON user_case_settings
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

-- Hot path on GET /cases and PATCH /cases/:id/settings.
CREATE INDEX user_case_settings_user_id_idx ON user_case_settings (user_id);
