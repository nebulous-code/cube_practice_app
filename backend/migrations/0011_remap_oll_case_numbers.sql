-- Remap OLL case_numbers to the Speedsolving canonical numbering. The
-- original numbering (migration 0003) was a personal/learning order that
-- doesn't match any external source, which makes it hard to cross-reference
-- with tutorials, the algorithm community, or other apps. The mapping below
-- is a bijection 1..57 → 1..57 derived from
-- docs/concepts/remap.csv (captured on 2026-05-12).
--
-- All other case-keyed data (user_case_settings, user_case_progress,
-- result_case_id pointers) references cases.id (UUID), so it follows the
-- row automatically — only the human-facing case_number column changes.
--
-- The UNIQUE (solve_stage_id, case_number) constraint forces a two-pass
-- update: phase 1 parks every OLL case_number in negative space so phase 2
-- can write the new positives without mid-flight collisions.
--
-- Note on migration 0008: that fix targeted "case 34 (Upstairs)" in the
-- pre-remap numbering. That physical row's result_rotation is preserved
-- across this migration; the row now has case_number = 36 in the canonical
-- numbering. See backend/tests/cases_seed.rs for the post-remap assertion.

-- ─── Phase 1: park in negative space ────────────────────────────────────────
UPDATE cases
   SET case_number = -case_number
 WHERE solve_stage_id = (SELECT id FROM solve_stages WHERE name = 'OLL');

-- ─── Phase 2: apply canonical mapping ───────────────────────────────────────
UPDATE cases c
   SET case_number = m.new_num
  FROM (VALUES
    ( 1,  1), ( 2,  2), ( 3,  3), ( 4,  4), ( 5, 19), ( 6, 18), ( 7, 17),
    ( 8, 55), ( 9, 52), (10, 51), (11, 56), (12, 48), (13, 47), (14, 49),
    (15, 50), (16, 53), (17, 54), (18,  9), (19, 10), (20,  6), (21,  5),
    (22,  7), (23,  8), (24, 12), (25, 11), (26, 14), (27, 13), (28, 16),
    (29, 15), (30, 20), (31, 35), (32, 37), (33, 38), (34, 36), (35, 32),
    (36, 31), (37, 44), (38, 43), (39, 29), (40, 30), (41, 41), (42, 42),
    (43, 40), (44, 39), (45, 46), (46, 34), (47, 45), (48, 33), (49, 22),
    (50, 21), (51, 27), (52, 26), (53, 57), (54, 28), (55, 23), (56, 24),
    (57, 25)
  ) AS m(old_num, new_num)
 WHERE c.solve_stage_id = (SELECT id FROM solve_stages WHERE name = 'OLL')
   AND c.case_number = -m.old_num;

-- ─── Sanity: confirm the mapping landed cleanly ─────────────────────────────
-- A typo in the VALUES list would either leave a row in negative space (no
-- match in phase 2) or duplicate a number (UNIQUE constraint catches it
-- mid-update). The block below is belt-and-suspenders: assert the final
-- shape is exactly 57 distinct values in 1..57.
DO $$
DECLARE
    out_of_range INT;
    distinct_count INT;
    total INT;
BEGIN
    SELECT COUNT(*) INTO total
      FROM cases
     WHERE solve_stage_id = (SELECT id FROM solve_stages WHERE name = 'OLL');

    SELECT COUNT(*) INTO out_of_range
      FROM cases
     WHERE solve_stage_id = (SELECT id FROM solve_stages WHERE name = 'OLL')
       AND (case_number < 1 OR case_number > 57);
    IF out_of_range > 0 THEN
        RAISE EXCEPTION 'OLL remap failed: % cases outside 1..57', out_of_range;
    END IF;

    SELECT COUNT(DISTINCT case_number) INTO distinct_count
      FROM cases
     WHERE solve_stage_id = (SELECT id FROM solve_stages WHERE name = 'OLL');
    IF distinct_count <> total THEN
        RAISE EXCEPTION
          'OLL remap failed: % distinct case_numbers across % rows', distinct_count, total;
    END IF;
END$$;
