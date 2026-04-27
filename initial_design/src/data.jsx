// OLL case data — parsed from the user's reference doc.
// Each case: { id, name, tags, algorithm, pattern (9-char string, row-major), result: {id, rotation 0-3} }
// rotation is in quarter-turns clockwise: 0 = same, 1 = 90° CW, 2 = 180°, 3 = 270° CW (= 90° CCW)

const OLL_CASES = [
  // Dot
  { id: 1, name: 'Tie Fighter', group: 'dot', priority: '*', algorithm: "R U (x') U' R U l' R' U' l' U l F'", pattern: 'LTRLXRLBR', result: { id: 2, rotation: 2 } },
  { id: 2, name: 'Dot Wall',    group: 'dot', priority: '*', algorithm: "F R U R' U' F' f R U R' U' f'",       pattern: 'LTTLXRLBB', result: { id: 2, rotation: 2 } },
  { id: 3, name: 'Left Pirate', group: 'dot', priority: '*', algorithm: "r' R2 U R' U r U2 r' U R' r",          pattern: 'TTRLXRXBB', result: { id: 4, rotation: 0 } },
  { id: 4, name: 'Right Pirate',group: 'dot', priority: '*', algorithm: "r' R U' r U2 r' U' R U' R2' r",        pattern: 'LTTLXRBBX', result: { id: 3, rotation: 0 } },
  { id: 5, name: 'Rocky',       group: 'dot', priority: '*', algorithm: "r' R U R U R' U' r R2' F R F'",        pattern: 'XTXLXRLBR', result: { id: 7, rotation: 2 } },
  { id: 6, name: 'Storage Unit',group: 'dot', priority: '*', algorithm: "F R U R' U (y') R' U2 R' F R F'",       pattern: 'TTTLXRXBX', result: { id: 7, rotation: 1 } },
  { id: 7, name: 'Diagonal',    group: 'dot', priority: '*', algorithm: "R U R' U R' F R F' U2 R' F R F'",       pattern: 'XTTLXRLBX', result: { id: 6, rotation: 2 } },
  { id: 30,name: 'Checkers',    group: 'dot', priority: '*', algorithm: "r' R U R U R' U' r2 R2' U R U' r'",     pattern: 'XTXLXRXBX', result: { id: 30, rotation: 0 } },

  // T-Shapes
  { id: 47,name: 'Overalls', group: 'T_shapes', priority: '-', algorithm: "F R U R' U' F'",                 pattern: 'LTXXXXLBX', result: { id: 37, rotation: 0 } },
  { id: 48,name: 'Slacks',   group: 'T_shapes', priority: '-', algorithm: "R U R' U' R' F R F'",            pattern: 'TTXXXXBBX', result: { id: 32, rotation: 0 } },

  // C-Shapes
  { id: 45,name: 'Magnet',   group: 'C_shapes', priority: '-', algorithm: "R' U' l' U l F' U R",            pattern: 'XXRLXRXXR', result: { id: 38, rotation: 0 } },
  { id: 46,name: 'Metroid',  group: 'C_shapes', priority: '-', algorithm: "R U R' U' (x) D' R' U R U' D (x')", pattern: 'LTRXXXXBX', result: { id: 32, rotation: 0 } },

  // Squares
  { id: 20,name: 'Right Dragon', group: 'squares', priority: 'L', algorithm: "r U2 R' U' R U' r'",            pattern: 'LXXLXXBBR', result: { id: 22, rotation: 0 } },
  { id: 21,name: 'Left Dragon',  group: 'squares', priority: 'L', algorithm: "l' U2 L U L' U l",              pattern: 'XXRXXRLBB', result: { id: 23, rotation: 0 } },

  // Lightning Bolts
  { id: 22,name: 'Tetris Right', group: 'lightning_bolts', priority: 'L', algorithm: "r U R' U R U2 r'",       pattern: 'TXRXXRXBB', result: { id: 20, rotation: 0 } },
  { id: 23,name: 'Tetris Left',  group: 'lightning_bolts', priority: 'L', algorithm: "l' U' L U' L' U2 l",     pattern: 'LXTLXXBBX', result: { id: 21, rotation: 0 } },
  { id: 24,name: 'Tetris Top',   group: 'lightning_bolts', priority: 'L', algorithm: "r R2' U' R U' R' U2 R U' R r'", pattern: 'XXTLXXBBR', result: { id: 21, rotation: 1 } },
  { id: 25,name: 'Tetris Bottom',group: 'lightning_bolts', priority: 'L', algorithm: "r' R2 U R' U R U2 R' U R' r",   pattern: 'TTRLXXXXB', result: { id: 20, rotation: 1 } },
  { id: 43,name: 'Right Pipe',   group: 'lightning_bolts', priority: '-', algorithm: "R' F R U R' U' F' U R",  pattern: 'XTTXXXLBX', result: { id: 36, rotation: 0 } },
  { id: 44,name: 'Left Pipe',    group: 'lightning_bolts', priority: '-', algorithm: "L F' L' U' L U F U' L'", pattern: 'TTXXXXXBR', result: { id: 35, rotation: 2 } },

  // I-Shapes
  { id: 8, name: 'Caterpillar', group: 'I_shapes', priority: '-', algorithm: "R' U2 R2 U R' U R U2 (x') U' R' U (x)", pattern: 'LXRLXRLXR', result: { id: 13, rotation: 2 } },
  { id: 9, name: 'Fridge',      group: 'I_shapes', priority: '-', algorithm: "R' U' R U' R' d R' U R B",     pattern: 'TXRLXRBXR', result: { id: 9, rotation: 1 } },
  { id: 10,name: 'Ant at Wall', group: 'I_shapes', priority: '-', algorithm: "f R U R' U' R U R' U' f'",     pattern: 'LTTXXXLBB', result: { id: 12, rotation: 2 } },
  { id: 11,name: 'Butterfly',   group: 'I_shapes', priority: '-', algorithm: "r' U' r U' R' U R U' R' U R r' U r", pattern: 'LTRXXXLBR', result: { id: 16, rotation: 1 } },

  // P-Shapes
  { id: 35,name: 'd Dot',  group: 'P_shapes', priority: 'L', algorithm: "R U B' U' R' U l U l'",          pattern: 'TTXLXXBXX', result: { id: 43, rotation: 2 } },
  { id: 36,name: 'q Dot',  group: 'P_shapes', priority: 'L', algorithm: "R' U' F U R U' R' F' R",         pattern: 'TXXLXXBBX', result: { id: 43, rotation: 0 } },
  { id: 37,name: 'p Wall', group: 'P_shapes', priority: 'L', algorithm: "F U R U' R' F'",                 pattern: 'XXRXXRXBR', result: { id: 47, rotation: 0 } },
  { id: 38,name: 'q Wall', group: 'P_shapes', priority: 'L', algorithm: "F' U' L' U L F",                 pattern: 'LXXLXXLBX', result: { id: 47, rotation: 2 } },

  // Small L
  { id: 12, name: null, group: 'small_L', priority: 'L', algorithm: "F R U R' U' R U R' U' F'",            pattern: 'LXTXXRLBB', result: { id: 10, rotation: 2 } },
  { id: 13, name: null, group: 'small_L', priority: 'L', algorithm: "F' L' U' L U L' U' L U F",           pattern: 'TXRLXXBBR', result: { id: 10, rotation: 0 } },
  { id: 14, name: null, group: 'small_L', priority: 'L', algorithm: "l' U R' U' R l U2 (x') U' R U l'",    pattern: 'LXTLXXLBB', result: { id: 17, rotation: 0 } },
  { id: 15, name: null, group: 'small_L', priority: 'L', algorithm: "R' F R2 B' R2' F' R2 B R'",           pattern: 'TXRXXRBBR', result: { id: 14, rotation: 2 } },
  { id: 16, name: null, group: 'small_L', priority: 'L', algorithm: "r' U' R U' R' U R U' R' U2 r",       pattern: 'LTRLXXLXR', result: { id: 16, rotation: 3 } },
  { id: 17, name: null, group: 'small_L', priority: 'L', algorithm: "r U R' U R U' R' U R U2' r'",        pattern: 'LXRLXXLBR', result: { id: 17, rotation: 1 } },

  // W-Shapes
  { id: 33,name: 'Basement Stairs', group: 'W_shapes', priority: 'L', algorithm: "R U R' U R U' R' U' R' F R F'", pattern: 'TXXXXRXBR', result: { id: 36, rotation: 3 } },
  { id: 34,name: 'Upstairs',        group: 'W_shapes', priority: 'L', algorithm: "R' U' R U' R' U R U l U' R' U (x)", pattern: 'XTRXXRBXX', result: { id: 35, rotation: 3 } },

  // Fish
  { id: 18,name: 'Down Boomerang', group: 'fish', priority: 'L', algorithm: "R' U' R (y' x') R U' R' F R U l'", pattern: 'LXTXXRBBX', result: { id: 27, rotation: 0 } },
  { id: 19,name: 'Up Boomerang',   group: 'fish', priority: 'L', algorithm: "R U R' (x z') R' U R B' R' U' l",  pattern: 'TTXXXRLXB', result: { id: 26, rotation: 0 } },
  { id: 31,name: 'Dot Kite',       group: 'fish', priority: 'L', algorithm: "R' U2 l R U' R' U l' U2' R",        pattern: 'TXXLXXXBR', result: { id: 32, rotation: 0 } },
  { id: 32,name: 'Stripe Kite',    group: 'fish', priority: 'L', algorithm: "F R U' R' U' R U R' F'",            pattern: 'XXRXXRBBX', result: { id: 48, rotation: 0 } },

  // Knight Moves
  { id: 26, name: null, group: 'knight_move', priority: '-', algorithm: "R' F R U l' U' l F U' F",              pattern: 'LTTXXXBBX', result: { id: 19, rotation: 1 } },
  { id: 27, name: null, group: 'knight_move', priority: '-', algorithm: "(x') R U' R' F' R U R' (x y) R' U R",   pattern: 'TTXXXXLBB', result: { id: 18, rotation: 0 } },
  { id: 28, name: null, group: 'knight_move', priority: '-', algorithm: "L F L' R U R' U' L F' L'",              pattern: 'LTXXXXBBR', result: { id: 22, rotation: 0 } },
  { id: 29, name: null, group: 'knight_move', priority: '-', algorithm: "L' B' L R' U' R U L' B L",              pattern: 'TTRXXXLBX', result: { id: 23, rotation: 2 } },

  // Awkward
  { id: 39,name: null, group: 'awkward_shape', priority: 'L', algorithm: "B' R B' R2' U R U R' U' R B2",         pattern: 'XTXXXRLXR', result: { id: 47, rotation: 0 } },
  { id: 40,name: null, group: 'awkward_shape', priority: 'L', algorithm: "R2' U R' B' R U' R2' U l U l'",        pattern: 'XTXLXXLXR', result: { id: 48, rotation: 0 } },
  { id: 41,name: null, group: 'awkward_shape', priority: 'L', algorithm: "R U R' U R U2' R' F R U R' U' F'",     pattern: 'TXTXXRXBX', result: { id: 38, rotation: 3 } },
  { id: 42,name: null, group: 'awkward_shape', priority: 'L', algorithm: "R' U' R U' R' U2 R F R U R' U' F'",    pattern: 'XTXXXRBXB', result: { id: 41, rotation: 0 } },

  // Corners correct
  { id: 53,name: 'Helipad',       group: 'corners_correct', priority: '-', algorithm: "R U R' U' r R' U R U' r'",      pattern: 'XTXXXXXBX', result: { id: 54, rotation: 2 } },
  { id: 54,name: 'Chipped Teeth', group: 'corners_correct', priority: 'L', algorithm: "r R' U R r' U2 r R' U R r'",    pattern: 'XTXLXXXXX', result: { id: 54, rotation: 1 } },

  // OCLL / Solves
  { id: 49,name: 'T-Shirt',   group: 'solves', priority: '+', algorithm: "R U2' R2' U' R2 U' R2' U2 R",   pattern: 'LXTXXXLXB', result: { id: 49, rotation: 0 } },
  { id: 50,name: 'Car',       group: 'solves', priority: '+', algorithm: "R U R' U R U' R' U R U2' R'",  pattern: 'LXRXXXLXR', result: { id: 50, rotation: 1 } },
  { id: 51,name: 'Spaceship', group: 'solves', priority: '+', algorithm: "R U R' U R U2' R'",            pattern: 'TXRXXXXXB', result: { id: 52, rotation: 0 } },
  { id: 52,name: 'Kickboxer', group: 'solves', priority: '+', algorithm: "R U2 R' U' R U' R'",           pattern: 'LXXXXXBXR', result: { id: 51, rotation: 0 } },
  { id: 55,name: 'Bird Flip', group: 'solves', priority: '+', algorithm: "R2' D R' U2 R D' R' U2 R'",    pattern: 'XXXXXXBXB', result: { id: 57, rotation: 2 } },
  { id: 56,name: 'Bull',      group: 'solves', priority: '+', algorithm: "l' U' L U R U' r' F",          pattern: 'XXTXXXXXB', result: { id: 57, rotation: 3 } },
  { id: 57,name: 'Dino',      group: 'solves', priority: '+', algorithm: "l' U' L' U R U' l U",          pattern: 'XXTXXXLXX', result: { id: 56, rotation: 0 } },
];

// Group display names
const GROUP_LABELS = {
  dot: 'Dot',
  T_shapes: 'T-Shapes',
  C_shapes: 'C-Shapes',
  squares: 'Squares',
  lightning_bolts: 'Lightning Bolts',
  I_shapes: 'I-Shapes',
  P_shapes: 'P-Shapes',
  small_L: 'Small L',
  W_shapes: 'W-Shapes',
  fish: 'Fish',
  knight_move: 'Knight Moves',
  awkward_shape: 'Awkward',
  corners_correct: 'Corners Correct',
  solves: 'OCLL / Solves',
};

const PRIORITY_LABELS = {
  '+': 'Known',
  '*': 'Dot — hardest',
  'L': 'Learning',
  '-': 'Not studying',
};

// Rotate pattern string (9 chars) and remap side-sticker letters
// L -> T -> R -> B -> L for each CW quarter turn
const SIDE_ROT = { L: 'T', T: 'R', R: 'B', B: 'L', X: 'X' };
function rotatePatternCW(p) {
  // positions: 0 1 2 / 3 4 5 / 6 7 8
  // After CW: new[r][c] = old[n-1-c][r]
  const idx = [6,3,0, 7,4,1, 8,5,2];
  let out = '';
  for (let i = 0; i < 9; i++) out += SIDE_ROT[p[idx[i]]];
  return out;
}
function rotatePattern(p, quarters) {
  let out = p;
  for (let i = 0; i < (quarters % 4 + 4) % 4; i++) out = rotatePatternCW(out);
  return out;
}

function caseById(id) {
  return OLL_CASES.find(c => c.id === id);
}

Object.assign(window, { OLL_CASES, GROUP_LABELS, PRIORITY_LABELS, rotatePattern, caseById });
