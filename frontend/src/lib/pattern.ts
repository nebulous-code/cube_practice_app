// 9-character OLL pattern utilities, ported from initial_design/src/data.jsx.
// See docs/milestones/02_case_data_and_browser.md §9 for the encoding.
//
// The string is a 3×3 grid (top-left to bottom-right). Each char encodes
// either a yellow top sticker (`X`) or a yellow side strip on the cubie's
// `T` / `L` / `R` / `B` face. Any other char is a non-yellow top sticker
// with no side flap.

const SIDE_ROT: Record<string, string> = {
  L: 'T',
  T: 'R',
  R: 'B',
  B: 'L',
  X: 'X',
}

// CW quarter-turn permutation of the 3×3 grid:
//   new[r][c] = old[n-1-c][r]
// Pre-computed lookup of the source index for each output index.
const CW_INDEX = [6, 3, 0, 7, 4, 1, 8, 5, 2]

export function rotatePatternCW(p: string): string {
  if (p.length !== 9) return p
  let out = ''
  for (let i = 0; i < 9; i++) {
    const idx = CW_INDEX[i]!
    const src = p.charAt(idx)
    out += SIDE_ROT[src] ?? src
  }
  return out
}

/// Rotate `pattern` by `quarters` clockwise turns. Negative or large values
/// are normalized into [0, 4).
export function rotatePattern(pattern: string, quarters: number): string {
  let out = pattern
  const n = ((quarters % 4) + 4) % 4
  for (let i = 0; i < n; i++) out = rotatePatternCW(out)
  return out
}
