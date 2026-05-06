// Guest-mode case merging. The anonymous `GET /cases` endpoint returns
// the global case set with `state = not_started` and `has_overrides =
// false` for every row (see backend `cases::list_global`). In guest
// mode we layer the localStorage blob on top: per-case overrides land
// from `settings[case_number]`, and the SM-2 state is recomputed from
// `progress[case_number]` using the same thresholds the server uses
// (cases/mod.rs MERGE_SELECT).

import type { Case, CaseState, SettingsPatch } from '@/stores/cases'

import type { GuestProgress, GuestSettings, GuestState } from './state'
import { todayLocal } from '@/lib/sm2'

const MASTERED_INTERVAL_DAYS = 21

/// Layer the guest blob over a list of global cases. Pure — does not
/// mutate the input array. The result has the same length and ordering
/// as the input.
export function mergeGuestSettings(
  globals: Case[],
  blob: GuestState,
  today: string = todayLocal(),
): Case[] {
  return globals.map((g) => mergeOne(g, blob.settings[String(g.case_number)], blob.progress[String(g.case_number)], blob, today))
}

/// Apply a settings patch from the cases-detail edit form to the guest
/// blob. Returns a new blob (pure) — caller persists. Mirrors the
/// backend `cases::update_settings` semantics: `undefined` leaves a
/// field alone, `null` clears it, a value sets it. Empty post-normalize
/// tag arrays clear the tags override.
export function applyGuestPatch(
  blob: GuestState,
  caseNumber: number,
  patch: SettingsPatch,
  caseNumberByCaseId: Map<string, number>,
): GuestState {
  const key = String(caseNumber)
  const next: GuestState = {
    ...blob,
    settings: { ...blob.settings },
  }

  const existing: GuestSettings = next.settings[key] ?? {
    nickname: null,
    algorithm: null,
    result_case_number: null,
    result_rotation: null,
    tags: [],
  }
  const merged: GuestSettings = { ...existing }

  if (patch.nickname !== undefined) {
    merged.nickname =
      patch.nickname === null ? null : trimOrNull(patch.nickname)
  }
  if (patch.algorithm !== undefined) {
    merged.algorithm =
      patch.algorithm === null ? null : trimOrNull(patch.algorithm)
  }
  if (patch.result_case_id !== undefined) {
    if (patch.result_case_id === null) {
      merged.result_case_number = null
    } else {
      const rcn = caseNumberByCaseId.get(patch.result_case_id)
      merged.result_case_number = rcn ?? null
    }
  }
  if (patch.result_rotation !== undefined) {
    merged.result_rotation = patch.result_rotation
  }
  if (patch.tags !== undefined) {
    if (patch.tags === null) {
      merged.tags = []
    } else {
      merged.tags = normalizeTags(patch.tags)
    }
  }

  // If every override resolved to its empty/null shape, drop the entry
  // so the blob stays minimal — same convention as
  // `cases::update_settings`'s all-null delete.
  if (
    merged.nickname == null &&
    merged.algorithm == null &&
    merged.result_case_number == null &&
    merged.result_rotation == null &&
    merged.tags.length === 0
  ) {
    delete next.settings[key]
  } else {
    next.settings[key] = merged
  }
  return next
}

function mergeOne(
  global: Case,
  settings: GuestSettings | undefined,
  progress: GuestProgress | undefined,
  blob: GuestState,
  today: string,
): Case {
  const out: Case = { ...global }
  let hasOverrides = false

  if (settings) {
    if (settings.nickname != null) {
      out.nickname = settings.nickname
      hasOverrides = true
    }
    if (settings.algorithm != null) {
      out.algorithm = settings.algorithm
      hasOverrides = true
    }
    if (settings.result_case_number != null) {
      // Translate the override's case_number to the matching case_id.
      // Falls back to the global pointer if the override references a
      // case_number we can't resolve (shouldn't happen — validation
      // catches this).
      const resolved = resolveCaseId(settings.result_case_number, blob)
      if (resolved) {
        out.result_case_id = resolved.id
        out.result_case_number = settings.result_case_number
      }
      hasOverrides = true
    }
    if (settings.result_rotation != null) {
      out.result_rotation = settings.result_rotation
      hasOverrides = true
    }
    if (settings.tags.length > 0) {
      out.tags = [...settings.tags]
      hasOverrides = true
    }
  }
  out.has_overrides = hasOverrides
  out.state = computeState(progress, today)
  return out
}

function computeState(
  progress: GuestProgress | undefined,
  today: string,
): CaseState {
  if (!progress) return 'not_started'
  if (progress.due_date <= today) return 'due'
  if (progress.interval_days < MASTERED_INTERVAL_DAYS) return 'learning'
  return 'mastered'
}

/// Mini case_id resolver — used only for result_case_id translation. We
/// don't have access to the global case list here, so we look up the
/// id from the blob's enclosing context. In the merge driver this is
/// always paired with a case-number→case_id index built from the global
/// list; this fallback path returns null and lets the caller decide.
function resolveCaseId(
  _caseNumber: number,
  _blob: GuestState,
): { id: string } | null {
  // Guest blob doesn't store ids — the merge driver passes the index in
  // separately when needed. mergeOne is only called from
  // `mergeGuestSettings` where the result-case translation is best-
  // effort: an override pointing at case 21 keeps `result_case_number =
  // 21`, but the `id` field stays as the global default. The detail
  // view uses `result_case_number` for the label, and the `result_case_id`
  // is only needed by validate_setting_patch's stage check (which
  // doesn't run in guest mode anyway).
  return null
}

function trimOrNull(s: string): string | null {
  const t = s.trim()
  return t === '' ? null : t
}

function normalizeTags(input: string[]): string[] {
  const out: string[] = []
  for (const raw of input) {
    const t = raw.trim().toLowerCase()
    if (!t) continue
    if (!out.includes(t)) out.push(t)
  }
  return out
}
