// Behavior tests for guest study helpers. The streak rule mirrors
// `backend/src/study/mod.rs`'s day-rollover logic; reviewCaseInGuest
// composes nextState() (already covered by sm2.spec.ts) with a streak
// update + blob mutation.

import { describe, expect, it } from 'vitest'

import { Grade } from '@/lib/sm2'

import { createInitialState } from './state'
import { dueQueueFromCases, reviewCaseInGuest, updateStreak } from './study-driver'
import type { Case } from '@/stores/cases'

describe('updateStreak', () => {
  it('null last_practice_date → starts at 1', () => {
    expect(updateStreak(null, '2026-05-12', 0)).toEqual({
      count: 1,
      last_practice_date: '2026-05-12',
    })
  })

  it('same day → unchanged count', () => {
    expect(updateStreak('2026-05-12', '2026-05-12', 7)).toEqual({
      count: 7,
      last_practice_date: '2026-05-12',
    })
  })

  it('yesterday → count + 1', () => {
    expect(updateStreak('2026-05-11', '2026-05-12', 7)).toEqual({
      count: 8,
      last_practice_date: '2026-05-12',
    })
  })

  it('two-day gap → resets to 1', () => {
    expect(updateStreak('2026-05-10', '2026-05-12', 7)).toEqual({
      count: 1,
      last_practice_date: '2026-05-12',
    })
  })
})

describe('reviewCaseInGuest', () => {
  it('first-ever review of a case writes a progress entry and starts streak', () => {
    const blob = createInitialState(new Date('2026-05-12T19:00:00Z'))
    const result = reviewCaseInGuest(blob, 12, Grade.Good, '2026-05-12')

    expect(result.blob.streak_count).toBe(1)
    expect(result.blob.last_practice_date).toBe('2026-05-12')
    expect(result.blob.progress['12']).toBeDefined()
    expect(result.progress.repetitions).toBe(1)
    expect(result.progress.interval_days).toBe(1)
    expect(result.progress.last_grade).toBe(Grade.Good)
  })

  it('second consecutive day Good → streak 2 and uses prior state', () => {
    const blob = createInitialState(new Date('2026-05-11T19:00:00Z'))
    const day1 = reviewCaseInGuest(blob, 12, Grade.Good, '2026-05-11')
    const day2 = reviewCaseInGuest(day1.blob, 12, Grade.Good, '2026-05-12')
    expect(day2.blob.streak_count).toBe(2)
    expect(day2.progress.repetitions).toBe(2)
    expect(day2.progress.interval_days).toBe(6)
  })

  it('fail resets the SM-2 fields but keeps the streak alive on the same day', () => {
    const blob = createInitialState(new Date('2026-05-11T19:00:00Z'))
    const day1 = reviewCaseInGuest(blob, 12, Grade.Good, '2026-05-11')
    // Same-day fail on a different case (or same): streak unchanged.
    const day1Fail = reviewCaseInGuest(day1.blob, 12, Grade.Fail, '2026-05-11')
    expect(day1Fail.blob.streak_count).toBe(1)
    expect(day1Fail.progress.repetitions).toBe(0)
    expect(day1Fail.progress.interval_days).toBe(1)
  })

  it('does not mutate the input blob', () => {
    const blob = createInitialState(new Date('2026-05-12T19:00:00Z'))
    const before = JSON.stringify(blob)
    reviewCaseInGuest(blob, 12, Grade.Good, '2026-05-12')
    expect(JSON.stringify(blob)).toBe(before)
  })
})

describe('dueQueueFromCases', () => {
  function caseWith(num: number, state: Case['state']): Case {
    return {
      id: `id${num}`,
      solve_stage: 'OLL',
      puzzle_type: '3x3',
      case_number: num,
      nickname: null,
      algorithm: 'R U R',
      result_case_id: null,
      result_case_number: null,
      result_rotation: 0,
      display_rotation: 0,
      pattern: 'XXXXXXXXX',
      tier1_tag: '+',
      tags: [],
      has_overrides: false,
      state,
    }
  }

  it('filters to due cases sorted by case_number', () => {
    const cases: Case[] = [
      caseWith(5, 'mastered'),
      caseWith(2, 'due'),
      caseWith(1, 'not_started'),
      caseWith(3, 'due'),
    ]
    const queue = dueQueueFromCases(cases)
    expect(queue.map((c) => c.case_number)).toEqual([2, 3])
  })
})
