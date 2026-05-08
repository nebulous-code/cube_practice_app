-- The 0003 seed shipped Case 34 (Upstairs) with result_rotation = 3 (90° CCW).
-- The correct rotation to land on Case 35 is 1 (90° CW). Confirmed against the
-- physical cube: applying Case 34's algorithm to a solved yellow top results in
-- Case 35's pattern rotated 90° clockwise from its canonical orientation.
--
-- We don't (and can't) modify 0003 in place — sqlx checksums applied migrations.
-- Idempotent: running this twice has no further effect.

UPDATE cases c
SET result_rotation = 1
WHERE c.case_number = 34
  AND c.solve_stage_id = (
      SELECT s.id
      FROM solve_stages s
      JOIN puzzle_types pt ON pt.id = s.puzzle_type_id
      WHERE s.name = 'OLL' AND pt.name = '3x3'
  );
