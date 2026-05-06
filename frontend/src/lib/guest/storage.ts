// localStorage adapter for the guest blob. One key, debounced writes,
// version-aware reads. See docs/milestones/06_guest_mode.md §5.
//
// Surface:
//   loadGuestState()  — read + migrate. Returns null when no blob, or
//                       when the blob is malformed (callers can surface
//                       a reset prompt; we don't auto-clear).
//   saveGuestState()  — debounced write. Coalesces rapid mutations so a
//                       chain of edits flushes once.
//   flushGuestState() — force-flush the pending write (e.g. before
//                       calling /upgrade so the latest blob makes the
//                       trip).
//   clearGuestState() — wipe the key. Used after successful upgrade
//                       and on the merge prompt's discard path.

import { migrateForward } from './migrate'
import type { GuestState } from './state'

export const STORAGE_KEY = 'oll-guest-state'
const DEBOUNCE_MS = 250

let pending: GuestState | null = null
let timer: ReturnType<typeof setTimeout> | null = null

/// Load + migrate. Returns null when storage is empty or the blob is
/// unrecoverable. Logs the migration error to the console so engineers
/// can debug — never throws.
export function loadGuestState(): GuestState | null {
  const raw = readRaw()
  if (raw === null) return null
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch (err) {
    console.warn('[guest] failed to parse stored blob — treating as empty', err)
    return null
  }
  const result = migrateForward(parsed)
  if (!result.ok) {
    console.warn(`[guest] migration failed: ${result.reason}`)
    return null
  }
  return result.state
}

/// Schedule a write. Subsequent calls within the debounce window
/// overwrite the pending state so the disk only sees the final value.
export function saveGuestState(state: GuestState): void {
  pending = state
  if (timer !== null) return
  timer = setTimeout(() => {
    flushGuestState()
  }, DEBOUNCE_MS)
}

/// Force any pending write to land synchronously.
export function flushGuestState(): void {
  if (timer !== null) {
    clearTimeout(timer)
    timer = null
  }
  if (pending === null) return
  writeRaw(JSON.stringify(pending))
  pending = null
}

/// Clear the blob and any pending write.
export function clearGuestState(): void {
  if (timer !== null) {
    clearTimeout(timer)
    timer = null
  }
  pending = null
  removeRaw()
}

// ─── Storage shim ───────────────────────────────────────────────────────────
// Wrapped so SSR / private-browsing scenarios where `localStorage` throws
// don't crash the app. Worst case the user practices a session and loses
// it on refresh — the banner already warns about that.

function readRaw(): string | null {
  try {
    return globalThis.localStorage?.getItem(STORAGE_KEY) ?? null
  } catch (err) {
    console.warn('[guest] localStorage read blocked', err)
    return null
  }
}

function writeRaw(value: string): void {
  try {
    globalThis.localStorage?.setItem(STORAGE_KEY, value)
  } catch (err) {
    console.warn('[guest] localStorage write blocked', err)
  }
}

function removeRaw(): void {
  try {
    globalThis.localStorage?.removeItem(STORAGE_KEY)
  } catch (err) {
    console.warn('[guest] localStorage remove blocked', err)
  }
}
