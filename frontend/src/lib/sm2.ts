// Anki-variant SM-2 update rule. TypeScript port of `backend/src/srs/mod.rs` —
// the Rust impl stays canonical. See docs/Cube_Practice_Design_Doc.md §4 and
// docs/milestones/03_core_study_loop.md §4.
//
// Used by the M6 guest-mode study driver: when running off localStorage we
// can't roundtrip to the backend for every grade, so the schedule update
// runs locally. Tests in sm2.spec.ts mirror the Rust test cases for parity.

export const INITIAL_EASE = 2.5
export const EASE_FLOOR = 1.3
export const HARD_INTERVAL_MULT = 1.2
export const EASY_BONUS = 1.3
export const FAIL_EASE_DELTA = -0.2
export const HARD_EASE_DELTA = -0.15
export const EASY_EASE_DELTA = 0.15

/// Grade values match the backend enum — Fail (0), Hard (1), Good (2), Easy (3).
export const Grade = {
  Fail: 0,
  Hard: 1,
  Good: 2,
  Easy: 3,
} as const
export type Grade = (typeof Grade)[keyof typeof Grade]

/// SM-2 row state. `due_date` is an ISO `YYYY-MM-DD` string — same shape
/// the localStorage blob and the API use.
export interface ProgressState {
  ease_factor: number
  interval_days: number
  repetitions: number
  due_date: string
}

/// Initial state for a brand-new card. `due_date` = today so the first
/// review treats today as the review day, matching the Rust impl.
export function initial(today: string): ProgressState {
  return {
    ease_factor: INITIAL_EASE,
    interval_days: 1,
    repetitions: 0,
    due_date: today,
  }
}

/// Apply the grade to `prev` using the Anki-variant SM-2 rule. Pure;
/// caller persists the returned state.
export function nextState(
  prev: ProgressState,
  grade: Grade,
  today: string,
): ProgressState {
  let ease_factor = prev.ease_factor
  let interval_days = prev.interval_days
  let repetitions = prev.repetitions

  if (grade === Grade.Fail) {
    repetitions = 0
    interval_days = 1
    ease_factor += FAIL_EASE_DELTA
  } else {
    if (repetitions === 0) {
      interval_days = 1
    } else if (repetitions === 1) {
      interval_days = 6
    } else if (grade === Grade.Hard) {
      interval_days = roundHalfAwayFromZero(interval_days * HARD_INTERVAL_MULT)
    } else if (grade === Grade.Good) {
      interval_days = roundHalfAwayFromZero(interval_days * ease_factor)
    } else if (grade === Grade.Easy) {
      interval_days = roundHalfAwayFromZero(interval_days * ease_factor * EASY_BONUS)
    }
    repetitions += 1
    if (grade === Grade.Hard) ease_factor += HARD_EASE_DELTA
    else if (grade === Grade.Easy) ease_factor += EASY_EASE_DELTA
    // Good leaves ease unchanged.
  }

  if (ease_factor < EASE_FLOOR) ease_factor = EASE_FLOOR
  if (interval_days < 1) interval_days = 1

  return {
    ease_factor,
    interval_days,
    repetitions,
    due_date: addDays(today, interval_days),
  }
}

// ─── Date helpers ───────────────────────────────────────────────────────────
//
// All inputs/outputs are `YYYY-MM-DD` strings. We parse + format in UTC to
// dodge DST and offset surprises — the SM-2 schedule is in calendar days,
// not wall-clock seconds, so timezone doesn't matter as long as we're
// consistent.

/// Add `days` whole days to an ISO date string and return another ISO date.
export function addDays(iso: string, days: number): string {
  const d = parseIsoDate(iso)
  d.setUTCDate(d.getUTCDate() + days)
  return formatIsoDate(d)
}

function parseIsoDate(iso: string): Date {
  const [yearStr, monthStr, dayStr] = iso.split('-')
  const year = Number(yearStr)
  const month = Number(monthStr)
  const day = Number(dayStr)
  if (!Number.isInteger(year) || !Number.isInteger(month) || !Number.isInteger(day)) {
    throw new Error(`Invalid ISO date: ${iso}`)
  }
  return new Date(Date.UTC(year, month - 1, day))
}

function formatIsoDate(d: Date): string {
  const y = d.getUTCFullYear().toString().padStart(4, '0')
  const m = (d.getUTCMonth() + 1).toString().padStart(2, '0')
  const day = d.getUTCDate().toString().padStart(2, '0')
  return `${y}-${m}-${day}`
}

/// Rust's `f64::round()` rounds half away from zero. JS's `Math.round`
/// rounds half toward positive infinity (`-0.5 → 0`, `0.5 → 1`). For
/// non-negative inputs the two agree, but mirror the Rust behavior
/// explicitly so future signed cases don't drift apart.
function roundHalfAwayFromZero(v: number): number {
  return v >= 0 ? Math.floor(v + 0.5) : -Math.floor(-v + 0.5)
}

/// Today as an ISO date string, evaluated in the local timezone. Guest
/// study uses local-day boundaries because that's how the user thinks
/// about their streak.
export function todayLocal(): string {
  const d = new Date()
  const y = d.getFullYear().toString().padStart(4, '0')
  const m = (d.getMonth() + 1).toString().padStart(2, '0')
  const day = d.getDate().toString().padStart(2, '0')
  return `${y}-${m}-${day}`
}
