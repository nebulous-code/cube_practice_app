// Parity tests for the SM-2 port. Mirrors the Rust tests in
// backend/src/srs/mod.rs so frontend and backend agree on every grade
// transition. New cases added on either side should land here too.

import { describe, expect, it } from 'vitest'

import {
  addDays,
  EASE_FLOOR,
  Grade,
  initial,
  INITIAL_EASE,
  nextState,
  type ProgressState,
} from './sm2'

const TODAY = '2026-04-29'

function atInitial(today: string): ProgressState {
  return initial(today)
}

// ─── Fail ────────────────────────────────────────────────────────────────────

describe('fail', () => {
  it('at rep 0 keeps rep 0', () => {
    const next = nextState(atInitial(TODAY), Grade.Fail, TODAY)
    expect(next.repetitions).toBe(0)
    expect(next.interval_days).toBe(1)
    expect(next.ease_factor).toBeCloseTo(2.3, 9)
    expect(next.due_date).toBe('2026-04-30')
  })

  it('after streak resets reps and interval', () => {
    const prev: ProgressState = {
      ease_factor: 2.4,
      interval_days: 30,
      repetitions: 5,
      due_date: '2026-05-29',
    }
    const next = nextState(prev, Grade.Fail, TODAY)
    expect(next.repetitions).toBe(0)
    expect(next.interval_days).toBe(1)
    expect(next.ease_factor).toBeCloseTo(2.2, 9)
    expect(next.due_date).toBe('2026-04-30')
  })

  it('does not drop ease below floor', () => {
    const prev: ProgressState = {
      ease_factor: EASE_FLOOR,
      interval_days: 3,
      repetitions: 2,
      due_date: '2026-05-02',
    }
    const next = nextState(prev, Grade.Fail, TODAY)
    expect(next.ease_factor).toBeCloseTo(EASE_FLOOR, 9)
  })
})

// ─── Good ────────────────────────────────────────────────────────────────────

describe('good', () => {
  it('at rep 0 advances to rep 1 interval 1', () => {
    const next = nextState(atInitial(TODAY), Grade.Good, TODAY)
    expect(next.repetitions).toBe(1)
    expect(next.interval_days).toBe(1)
    expect(next.ease_factor).toBeCloseTo(INITIAL_EASE, 9)
    expect(next.due_date).toBe('2026-04-30')
  })

  it('at rep 1 advances to rep 2 interval 6', () => {
    const prev: ProgressState = {
      ease_factor: 2.5,
      interval_days: 1,
      repetitions: 1,
      due_date: '2026-04-30',
    }
    const next = nextState(prev, Grade.Good, '2026-04-30')
    expect(next.repetitions).toBe(2)
    expect(next.interval_days).toBe(6)
    expect(next.ease_factor).toBeCloseTo(2.5, 9)
    expect(next.due_date).toBe('2026-05-06')
  })

  it('at rep 2 multiplies by ease', () => {
    const prev: ProgressState = {
      ease_factor: 2.5,
      interval_days: 6,
      repetitions: 2,
      due_date: '2026-05-06',
    }
    const next = nextState(prev, Grade.Good, '2026-05-06')
    expect(next.repetitions).toBe(3)
    expect(next.interval_days).toBe(15) // round(6 * 2.5)
    expect(next.ease_factor).toBeCloseTo(2.5, 9)
    expect(next.due_date).toBe('2026-05-21')
  })
})

// ─── Hard ────────────────────────────────────────────────────────────────────

describe('hard', () => {
  it('at rep 0 acts like first review with ease drop', () => {
    const next = nextState(atInitial(TODAY), Grade.Hard, TODAY)
    expect(next.repetitions).toBe(1)
    expect(next.interval_days).toBe(1)
    expect(next.ease_factor).toBeCloseTo(2.35, 9)
  })

  it('at rep 2 uses hard multiplier and ease drops', () => {
    const prev: ProgressState = {
      ease_factor: 2.5,
      interval_days: 10,
      repetitions: 2,
      due_date: '2026-05-09',
    }
    const next = nextState(prev, Grade.Hard, '2026-05-09')
    expect(next.repetitions).toBe(3)
    expect(next.interval_days).toBe(12) // round(10 * 1.2)
    expect(next.ease_factor).toBeCloseTo(2.35, 9)
  })
})

// ─── Easy ────────────────────────────────────────────────────────────────────

describe('easy', () => {
  it('at rep 2 applies easy bonus and ease rises', () => {
    const prev: ProgressState = {
      ease_factor: 2.5,
      interval_days: 10,
      repetitions: 2,
      due_date: '2026-05-09',
    }
    const next = nextState(prev, Grade.Easy, '2026-05-09')
    expect(next.repetitions).toBe(3)
    expect(next.interval_days).toBe(33) // round(10 * 2.5 * 1.3) = round(32.5) = 33
    expect(next.ease_factor).toBeCloseTo(2.65, 9)
  })
})

// ─── Floor + interval guard ──────────────────────────────────────────────────

describe('ease floor', () => {
  it('holds through consecutive hards', () => {
    let state: ProgressState = {
      ease_factor: 1.4,
      interval_days: 4,
      repetitions: 3,
      due_date: '2026-05-03',
    }
    state = nextState(state, Grade.Hard, '2026-05-03')
    expect(state.ease_factor).toBeCloseTo(EASE_FLOOR, 9)
    state = nextState(state, Grade.Hard, '2026-05-03')
    expect(state.ease_factor).toBeCloseTo(EASE_FLOOR, 9)
  })
})

// ─── Round trip ──────────────────────────────────────────────────────────────

describe('round trip', () => {
  it('good good good matches expected sequence', () => {
    let today = '2026-04-29'
    let state = initial(today)

    state = nextState(state, Grade.Good, today)
    expect([state.repetitions, state.interval_days]).toEqual([1, 1])
    today = state.due_date

    state = nextState(state, Grade.Good, today)
    expect([state.repetitions, state.interval_days]).toEqual([2, 6])
    today = state.due_date

    state = nextState(state, Grade.Good, today)
    expect([state.repetitions, state.interval_days]).toEqual([3, 15])
  })
})

// ─── Date helpers ────────────────────────────────────────────────────────────

describe('addDays', () => {
  it('adds whole days', () => {
    expect(addDays('2026-04-29', 1)).toBe('2026-04-30')
    expect(addDays('2026-04-29', 7)).toBe('2026-05-06')
  })

  it('crosses month boundaries', () => {
    expect(addDays('2026-04-29', 2)).toBe('2026-05-01')
    expect(addDays('2026-12-31', 1)).toBe('2027-01-01')
  })

  it('crosses leap-year February correctly', () => {
    expect(addDays('2024-02-28', 1)).toBe('2024-02-29')
    expect(addDays('2024-02-29', 1)).toBe('2024-03-01')
    expect(addDays('2025-02-28', 1)).toBe('2025-03-01')
  })
})
