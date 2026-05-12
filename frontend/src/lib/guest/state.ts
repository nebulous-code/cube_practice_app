// Typed schema for the guest-mode localStorage blob. Mirrors the Rust
// `GuestState` in `backend/src/guest_state/mod.rs`. Schema is versioned
// so the on-device shape can evolve forward without breaking older
// installs — see `migrate.ts`. Current version is 1.

export const SCHEMA_VERSION = 1 as const

/// Per-case override row. Fields default to undefined / empty when the
/// user hasn't customized them.
export interface GuestSettings {
  nickname?: string | null
  algorithm?: string | null
  result_case_number?: number | null
  result_rotation?: number | null
  display_rotation?: number | null
  tags: string[]
}

/// Per-case SM-2 progress row. Only present once a case has been graded
/// at least once — `not_started` cases don't appear here.
export interface GuestProgress {
  ease_factor: number
  interval_days: number
  repetitions: number
  due_date: string // YYYY-MM-DD
  last_grade?: number | null
  last_reviewed?: string | null // ISO 8601 timestamp
}

/// Top-level guest blob. Keys of `settings` and `progress` are case_number
/// (1–57) as a string — matches the JSON shape the backend accepts.
export interface GuestState {
  version: typeof SCHEMA_VERSION
  display_name: string | null
  created_at: string // ISO 8601 timestamp
  streak_count: number
  last_practice_date: string | null // YYYY-MM-DD
  onboarding_completed: boolean
  banner_dismissed_at: string | null // ISO 8601 timestamp
  banner_suppressed_until_reviews: number | null
  settings: Record<string, GuestSettings>
  progress: Record<string, GuestProgress>
}

/// Build a fresh, empty guest blob. Called when a visitor first taps
/// "Continue as guest" — the splash check finds no existing blob and
/// seeds this one before routing to the onboarding stub.
export function createInitialState(now: Date = new Date()): GuestState {
  return {
    version: SCHEMA_VERSION,
    display_name: null,
    created_at: now.toISOString(),
    streak_count: 0,
    last_practice_date: null,
    onboarding_completed: false,
    banner_dismissed_at: null,
    banner_suppressed_until_reviews: null,
    settings: {},
    progress: {},
  }
}
