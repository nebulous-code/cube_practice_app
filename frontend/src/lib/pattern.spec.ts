import { describe, expect, it } from 'vitest'

import { rotatePattern, rotatePatternCW } from './pattern'

describe('rotatePatternCW', () => {
  it('returns the input when length is not 9', () => {
    expect(rotatePatternCW('XXX')).toBe('XXX')
  })

  it('rotates positions 90° clockwise', () => {
    // Index map: input index → output index after CW rotation.
    // Input '012345678' → output should pull from indices [6,3,0,7,4,1,8,5,2].
    // Use chars that aren't side-strip letters so SIDE_ROT passes them through.
    expect(rotatePatternCW('012345678')).toBe('630741852')
  })

  it('rotates side-strip letters L→T→R→B→L', () => {
    // Each cubie position holds a different side letter so we can see the rotation.
    // Input arrangement (positions 0..8): L T R L X R L B R
    // After CW: positions [6,3,0,7,4,1,8,5,2] of input = L L L B X T R R R
    // After SIDE_ROT (L→T, T→R, R→B, B→L, X→X): T T T L X R B B B
    expect(rotatePatternCW('LTRLXRLBR')).toBe('TTTLXRBBB')
  })

  it('keeps X (top yellow) unchanged when rotating', () => {
    expect(rotatePatternCW('XXXXXXXXX')).toBe('XXXXXXXXX')
  })
})

describe('rotatePattern', () => {
  it('zero turns is identity', () => {
    expect(rotatePattern('LTRLXRLBR', 0)).toBe('LTRLXRLBR')
  })

  it('four turns returns the original', () => {
    const p = 'LTRLXRLBR'
    expect(rotatePattern(p, 4)).toBe(p)
  })

  it('two turns equals two CW applications', () => {
    const once = rotatePatternCW('LTRLXRLBR')
    const twice = rotatePatternCW(once)
    expect(rotatePattern('LTRLXRLBR', 2)).toBe(twice)
  })

  it('normalizes negative quarter values', () => {
    // -1 should equal 3 CW (= 1 CCW).
    expect(rotatePattern('LTRLXRLBR', -1)).toBe(rotatePattern('LTRLXRLBR', 3))
  })

  it('normalizes large quarter values', () => {
    expect(rotatePattern('LTRLXRLBR', 5)).toBe(rotatePattern('LTRLXRLBR', 1))
  })
})
