// Forward-version migrations for the guest blob (Q6 — frontend on load).
// v1 is the only schema today; this module exists so future bumps have an
// obvious place to live. The bootstrap path runs `migrateForward` on
// every read; if the loaded blob is at the latest version it short-
// circuits.

import { type GuestState, SCHEMA_VERSION } from './state'

/// Result of a forward migration. `null` means the blob was unrecoverable
/// (unknown version from the future, or completely garbled) — callers
/// treat that the same as "no blob present" and offer the user to
/// discard or start over.
export type MigrateResult =
  | { ok: true; state: GuestState }
  | { ok: false; reason: string }

export function migrateForward(raw: unknown): MigrateResult {
  if (typeof raw !== 'object' || raw === null) {
    return { ok: false, reason: 'Not a JSON object.' }
  }
  const blob = raw as { version?: unknown }
  const version = typeof blob.version === 'number' ? blob.version : null
  if (version === null) {
    return { ok: false, reason: 'Missing version field.' }
  }

  // Forward-only — we never migrate backward. A blob from a newer client
  // is a recipe for data loss if we tried to massage it down.
  if (version > SCHEMA_VERSION) {
    return {
      ok: false,
      reason: `Blob version ${version} is newer than client (${SCHEMA_VERSION}).`,
    }
  }

  if (version === SCHEMA_VERSION) {
    return { ok: true, state: raw as GuestState }
  }

  // No prior versions exist yet. When v2 lands, replace the unreachable
  // with a step-wise migration: v1 → v2 → … → SCHEMA_VERSION.
  return { ok: false, reason: `No migration path from version ${version}.` }
}
