-- Seed the 3×3 puzzle type, the OLL stage, and all 57 OLL cases.
-- Source of truth: initial_design/src/data.jsx (confirmed correct in
-- docs/outstanding_decision.md §2.6).
--
-- Idempotent: re-running this migration on an existing database upserts
-- every column and rebuilds the result_case_id mapping. This matters
-- because Render runs migrations on every deploy.
--
-- Algorithms use dollar quoting ($$...$$) to avoid having to double every
-- prime (apostrophe) in SQL.

-- ─── Step 1: puzzle type ─────────────────────────────────────────────────────

INSERT INTO puzzle_types (name)
VALUES ('3x3')
ON CONFLICT (name) DO NOTHING;

-- ─── Step 2: solve stage ─────────────────────────────────────────────────────

INSERT INTO solve_stages (puzzle_type_id, name, description, display_order)
SELECT pt.id, 'OLL', 'Orientation of the Last Layer', 0
FROM puzzle_types pt
WHERE pt.name = '3x3'
ON CONFLICT (puzzle_type_id, name) DO NOTHING;

-- ─── Step 3: cases (without result_case_id; backfilled in step 4) ────────────

INSERT INTO cases
    (solve_stage_id, case_number, nickname, algorithm, diagram_data,
     tier1_tag, tier2_tag, result_rotation)
SELECT s.id, v.case_number, v.nickname, v.algorithm,
       jsonb_build_object('pattern', v.pattern),
       v.tier1_tag, v.tier2_tag, v.result_rotation
FROM solve_stages s
JOIN puzzle_types pt ON pt.id = s.puzzle_type_id
CROSS JOIN (VALUES
    -- Dot
    (1::int,  'Tie Fighter'::text,    $$R U (x') U' R U l' R' U' l' U l F'$$,         'LTRLXRLBR', '*', 'dot', 2::int),
    (2,       'Dot Wall',              $$F R U R' U' F' f R U R' U' f'$$,             'LTTLXRLBB', '*', 'dot', 2),
    (3,       'Left Pirate',           $$r' R2 U R' U r U2 r' U R' r$$,                'TTRLXRXBB', '*', 'dot', 0),
    (4,       'Right Pirate',          $$r' R U' r U2 r' U' R U' R2' r$$,              'LTTLXRBBX', '*', 'dot', 0),
    (5,       'Rocky',                 $$r' R U R U R' U' r R2' F R F'$$,              'XTXLXRLBR', '*', 'dot', 2),
    (6,       'Storage Unit',          $$F R U R' U (y') R' U2 R' F R F'$$,            'TTTLXRXBX', '*', 'dot', 1),
    (7,       'Diagonal',              $$R U R' U R' F R F' U2 R' F R F'$$,            'XTTLXRLBX', '*', 'dot', 2),
    (30,      'Checkers',              $$r' R U R U R' U' r2 R2' U R U' r'$$,          'XTXLXRXBX', '*', 'dot', 0),

    -- T-Shapes
    (47,      'Overalls',              $$F R U R' U' F'$$,                             'LTXXXXLBX', '-', 'T_shapes', 0),
    (48,      'Slacks',                $$R U R' U' R' F R F'$$,                        'TTXXXXBBX', '-', 'T_shapes', 0),

    -- C-Shapes
    (45,      'Magnet',                $$R' U' l' U l F' U R$$,                        'XXRLXRXXR', '-', 'C_shapes', 0),
    (46,      'Metroid',               $$R U R' U' (x) D' R' U R U' D (x')$$,          'LTRXXXXBX', '-', 'C_shapes', 0),

    -- Squares
    (20,      'Right Dragon',          $$r U2 R' U' R U' r'$$,                         'LXXLXXBBR', 'L', 'squares', 0),
    (21,      'Left Dragon',           $$l' U2 L U L' U l$$,                           'XXRXXRLBB', 'L', 'squares', 0),

    -- Lightning Bolts
    (22,      'Tetris Right',          $$r U R' U R U2 r'$$,                           'TXRXXRXBB', 'L', 'lightning_bolts', 0),
    (23,      'Tetris Left',           $$l' U' L U' L' U2 l$$,                         'LXTLXXBBX', 'L', 'lightning_bolts', 0),
    (24,      'Tetris Top',            $$r R2' U' R U' R' U2 R U' R r'$$,              'XXTLXXBBR', 'L', 'lightning_bolts', 1),
    (25,      'Tetris Bottom',         $$r' R2 U R' U R U2 R' U R' r$$,                'TTRLXXXXB', 'L', 'lightning_bolts', 1),
    (43,      'Right Pipe',            $$R' F R U R' U' F' U R$$,                      'XTTXXXLBX', '-', 'lightning_bolts', 0),
    (44,      'Left Pipe',             $$L F' L' U' L U F U' L'$$,                     'TTXXXXXBR', '-', 'lightning_bolts', 2),

    -- I-Shapes
    (8,       'Caterpillar',           $$R' U2 R2 U R' U R U2 (x') U' R' U (x)$$,      'LXRLXRLXR', '-', 'I_shapes', 2),
    (9,       'Fridge',                $$R' U' R U' R' d R' U R B$$,                   'TXRLXRBXR', '-', 'I_shapes', 1),
    (10,      'Ant at Wall',           $$f R U R' U' R U R' U' f'$$,                   'LTTXXXLBB', '-', 'I_shapes', 2),
    (11,      'Butterfly',             $$r' U' r U' R' U R U' R' U R r' U r$$,         'LTRXXXLBR', '-', 'I_shapes', 1),

    -- P-Shapes
    (35,      'd Dot',                 $$R U B' U' R' U l U l'$$,                      'TTXLXXBXX', 'L', 'P_shapes', 2),
    (36,      'q Dot',                 $$R' U' F U R U' R' F' R$$,                     'TXXLXXBBX', 'L', 'P_shapes', 0),
    (37,      'p Wall',                $$F U R U' R' F'$$,                             'XXRXXRXBR', 'L', 'P_shapes', 0),
    (38,      'q Wall',                $$F' U' L' U L F$$,                             'LXXLXXLBX', 'L', 'P_shapes', 2),

    -- Small L
    (12,      NULL,                    $$F R U R' U' R U R' U' F'$$,                   'LXTXXRLBB', 'L', 'small_L', 2),
    (13,      NULL,                    $$F' L' U' L U L' U' L U F$$,                   'TXRLXXBBR', 'L', 'small_L', 0),
    (14,      NULL,                    $$l' U R' U' R l U2 (x') U' R U l'$$,           'LXTLXXLBB', 'L', 'small_L', 0),
    (15,      NULL,                    $$R' F R2 B' R2' F' R2 B R'$$,                  'TXRXXRBBR', 'L', 'small_L', 2),
    (16,      NULL,                    $$r' U' R U' R' U R U' R' U2 r$$,               'LTRLXXLXR', 'L', 'small_L', 3),
    (17,      NULL,                    $$r U R' U R U' R' U R U2' r'$$,                'LXRLXXLBR', 'L', 'small_L', 1),

    -- W-Shapes
    (33,      'Basement Stairs',       $$R U R' U R U' R' U' R' F R F'$$,              'TXXXXRXBR', 'L', 'W_shapes', 3),
    (34,      'Upstairs',              $$R' U' R U' R' U R U l U' R' U (x)$$,          'XTRXXRBXX', 'L', 'W_shapes', 3),

    -- Fish
    (18,      'Down Boomerang',        $$R' U' R (y' x') R U' R' F R U l'$$,           'LXTXXRBBX', 'L', 'fish', 0),
    (19,      'Up Boomerang',          $$R U R' (x z') R' U R B' R' U' l$$,            'TTXXXRLXB', 'L', 'fish', 0),
    (31,      'Dot Kite',              $$R' U2 l R U' R' U l' U2' R$$,                 'TXXLXXXBR', 'L', 'fish', 0),
    (32,      'Stripe Kite',           $$F R U' R' U' R U R' F'$$,                     'XXRXXRBBX', 'L', 'fish', 0),

    -- Knight Moves
    (26,      NULL,                    $$R' F R U l' U' l F U' F$$,                    'LTTXXXBBX', '-', 'knight_move', 1),
    (27,      NULL,                    $$(x') R U' R' F' R U R' (x y) R' U R$$,        'TTXXXXLBB', '-', 'knight_move', 0),
    (28,      NULL,                    $$L F L' R U R' U' L F' L'$$,                   'LTXXXXBBR', '-', 'knight_move', 0),
    (29,      NULL,                    $$L' B' L R' U' R U L' B L$$,                   'TTRXXXLBX', '-', 'knight_move', 2),

    -- Awkward
    (39,      NULL,                    $$B' R B' R2' U R U R' U' R B2$$,               'XTXXXRLXR', 'L', 'awkward_shape', 0),
    (40,      NULL,                    $$R2' U R' B' R U' R2' U l U l'$$,              'XTXLXXLXR', 'L', 'awkward_shape', 0),
    (41,      NULL,                    $$R U R' U R U2' R' F R U R' U' F'$$,           'TXTXXRXBX', 'L', 'awkward_shape', 3),
    (42,      NULL,                    $$R' U' R U' R' U2 R F R U R' U' F'$$,          'XTXXXRBXB', 'L', 'awkward_shape', 0),

    -- Corners correct
    (53,      'Helipad',               $$R U R' U' r R' U R U' r'$$,                   'XTXXXXXBX', '-', 'corners_correct', 2),
    (54,      'Chipped Teeth',         $$r R' U R r' U2 r R' U R r'$$,                 'XTXLXXXXX', 'L', 'corners_correct', 1),

    -- OCLL / Solves
    (49,      'T-Shirt',               $$R U2' R2' U' R2 U' R2' U2 R$$,                'LXTXXXLXB', '+', 'solves', 0),
    (50,      'Car',                   $$R U R' U R U' R' U R U2' R'$$,                'LXRXXXLXR', '+', 'solves', 1),
    (51,      'Spaceship',             $$R U R' U R U2' R'$$,                          'TXRXXXXXB', '+', 'solves', 0),
    (52,      'Kickboxer',             $$R U2 R' U' R U' R'$$,                         'LXXXXXBXR', '+', 'solves', 0),
    (55,      'Bird Flip',             $$R2' D R' U2 R D' R' U2 R'$$,                  'XXXXXXBXB', '+', 'solves', 2),
    (56,      'Bull',                  $$l' U' L U R U' r' F$$,                        'XXTXXXXXB', '+', 'solves', 3),
    (57,      'Dino',                  $$l' U' L' U R U' l U$$,                        'XXTXXXLXX', '+', 'solves', 0)
) AS v(case_number, nickname, algorithm, pattern, tier1_tag, tier2_tag, result_rotation)
WHERE s.name = 'OLL' AND pt.name = '3x3'
ON CONFLICT (solve_stage_id, case_number) DO UPDATE SET
    nickname        = EXCLUDED.nickname,
    algorithm       = EXCLUDED.algorithm,
    diagram_data    = EXCLUDED.diagram_data,
    tier1_tag       = EXCLUDED.tier1_tag,
    tier2_tag       = EXCLUDED.tier2_tag,
    result_rotation = EXCLUDED.result_rotation;

-- ─── Step 4: backfill result_case_id (now that all 57 rows exist) ────────────

UPDATE cases c
SET result_case_id = target.id
FROM (VALUES
    (1::int,  2::int),
    (2,       2),
    (3,       4),
    (4,       3),
    (5,       7),
    (6,       7),
    (7,       6),
    (8,       13),
    (9,       9),
    (10,      12),
    (11,      16),
    (12,      10),
    (13,      10),
    (14,      17),
    (15,      14),
    (16,      16),
    (17,      17),
    (18,      27),
    (19,      26),
    (20,      22),
    (21,      23),
    (22,      20),
    (23,      21),
    (24,      21),
    (25,      20),
    (26,      19),
    (27,      18),
    (28,      22),
    (29,      23),
    (30,      30),
    (31,      32),
    (32,      48),
    (33,      36),
    (34,      35),
    (35,      43),
    (36,      43),
    (37,      47),
    (38,      47),
    (39,      47),
    (40,      48),
    (41,      38),
    (42,      41),
    (43,      36),
    (44,      35),
    (45,      38),
    (46,      32),
    (47,      37),
    (48,      32),
    (49,      49),
    (50,      50),
    (51,      52),
    (52,      51),
    (53,      54),
    (54,      54),
    (55,      57),
    (56,      57),
    (57,      56)
) AS r(case_number, result_case_number),
solve_stages s,
puzzle_types pt,
cases target
WHERE c.case_number       = r.case_number
  AND c.solve_stage_id    = s.id
  AND s.name              = 'OLL'
  AND pt.id               = s.puzzle_type_id
  AND pt.name             = '3x3'
  AND target.case_number  = r.result_case_number
  AND target.solve_stage_id = s.id;
