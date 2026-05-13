// Guest-mode study driver. The authenticated path POSTs to
// `/study/:id/review` and reads the new state back; in guest mode we run
// SM-2 client-side (`@/lib/sm2`) and write the result straight to the
// localStorage blob. See docs/milestones/06_guest_mode.md §5.
//
// All functions here are pure — they take the current blob shape and
// return either a new blob or a derived value. The store is responsible
// for persisting the new blob via `auth.updateGuestState()`.

import { addDays, type Grade, initial, nextState, todayLocal } from '@/lib/sm2'
import type { Case } from '@/stores/cases'

import type { GuestProgress, GuestState } from './state'

export interface ReviewResult {
  blob: GuestState
  /// SM-2 fields after the review, in the same shape the API would
  /// return. Drivers map this back into the `Case` cache.
  progress: GuestProgress
}

/// Apply a grade to a case in guest mode. Pure — returns the new blob
/// alongside the post-review progress row.
export function reviewCaseInGuest(
  blob: GuestState,
  caseNumber: number,
  grade: Grade,
  today: string = todayLocal(),
): ReviewResult {
  const key = String(caseNumber)
  const prevState = blob.progress[key]
  const prev = prevState
    ? {
        ease_factor: prevState.ease_factor,
        interval_days: prevState.interval_days,
        repetitions: prevState.repetitions,
        due_date: prevState.due_date,
      }
    : initial(today)

  const next = nextState(prev, grade, today)
  const progress: GuestProgress = {
    ease_factor: next.ease_factor,
    interval_days: next.interval_days,
    repetitions: next.repetitions,
    due_date: next.due_date,
    last_grade: grade,
    last_reviewed: new Date().toISOString(),
  }

  const streak = updateStreak(blob.last_practice_date, today, blob.streak_count)

  const updated: GuestState = {
    ...blob,
    progress: { ...blob.progress, [key]: progress },
    streak_count: streak.count,
    last_practice_date: streak.last_practice_date,
  }
  return { blob: updated, progress }
}

/// Day-rollover streak rule, mirroring `backend/src/study/mod.rs`:
///   - last_practice_date == null → streak = 1
///   - last_practice_date == today → unchanged
///   - last_practice_date == today - 1 → streak += 1
///   - else → streak = 1
/// Always sets last_practice_date = today.
export function updateStreak(
  last: string | null,
  today: string,
  current: number,
): { count: number; last_practice_date: string } {
  if (last === null) return { count: 1, last_practice_date: today }
  if (last === today) return { count: current, last_practice_date: today }
  const yesterday = addDays(today, -1)
  if (last === yesterday) {
    return { count: current + 1, last_practice_date: today }
  }
  return { count: 1, last_practice_date: today }
}

/// Pull the due queue for guest mode out of the merged cases list. The
/// merge runs in the cases store, so by the time we get here every Case
/// already has its computed `state`. We just filter + sort.
export function dueQueueFromCases(cases: Case[]): Case[] {
  return cases
    .filter((c) => c.state === 'due')
    .slice()
    .sort((a, b) => a.case_number - b.case_number)
}
