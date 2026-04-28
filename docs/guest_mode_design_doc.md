# Guest Mode — Design Document

Guest mode lets a new visitor practice OLL without creating an account, with progress stored on-device in `localStorage`. It ships as the **final MVP feature** so that the rest of the app is built against the authenticated path first; guest mode is layered on top once the auth-backed flows are stable.

This document captures the design before any other code is written so that decisions made earlier in the build (schema, API shapes, study state, streak logic) leave room for it without retrofits.

---

## 1. Why a separate document

The main spec describes a fully-authenticated app. Guest mode introduces a second persistence backend (browser `localStorage`) and a migration step (guest data → server account). Embedding that branching throughout the main spec would obscure the simple authenticated path. Keeping it isolated here lets the implementation read as "everything in the main spec, plus this layer."

---

## 2. Entry points

A user can enter guest mode via:

- The **"Continue as guest"** button on `LoginScreen` (`screen-auth.jsx:56-63`)
- A direct link to a guest-mode URL (TBD — see open question 1)

A guest can leave guest mode in two ways:

- **Sign in** to an existing account — guest progress is *not* migrated; it stays on the device
- **Create an account** via the `GuestUpgradeScreen` — guest progress *is* migrated into the new account (see §6)

A guest can also clear their data by signing out / clearing browser storage — this destroys all guest progress with no recovery.

---

## 3. What guest mode supports vs doesn't

| Feature | Guest | Authenticated |
|---------|:-----:|:-------------:|
| Practice / Free study | ✓ | ✓ |
| 4-button grading + scheduling | ✓ | ✓ |
| Per-case overrides (algorithm, nickname, result) | ✓ | ✓ |
| User-defined free-form tags | ✓ | ✓ |
| Streak tracking | ✓ | ✓ |
| Progress dashboard | ✓ | ✓ |
| Email-related flows (verify, reset, change) | — | ✓ |
| Cross-device sync | — | ✓ |
| `Sign out everywhere` | — | ✓ |
| Settings → display name + email edit | partial — display name only | ✓ |
| Onboarding | ✓ (one-time, same as authed) | ✓ |

Anything that requires a server-side identity is unavailable to guests. Everything else has full parity.

---

## 4. Data shape in localStorage

A single key `oll-guest-state` holds a JSON blob shaped like this:

```jsonc
{
  "version": 1,                         // schema version for forward migrations
  "display_name": "Nick",               // user-set, optional, default null
  "created_at": "2026-04-28T20:13:00Z", // first time guest mode was entered on this device
  "streak_count": 7,
  "last_practice_date": "2026-04-28",
  "settings": {
    // keyed by case_number (1–57); only entries with overrides are present
    "12": { "nickname": "Slash", "algorithm": "F R U R' U' F'", "result_case_number": 21, "result_rotation": 1, "tier2_tag": "small_L" }
  },
  "progress": {
    // keyed by case_number; only entries with at least one review
    "12": {
      "ease_factor": 2.5,
      "interval_days": 6,
      "repetitions": 2,
      "due_date": "2026-05-04",
      "last_grade": 2,
      "last_reviewed": "2026-04-28T20:11:23Z"
    }
  },
  "tags": [
    { "id": "g_1", "name": "needs work" }   // guest-only IDs are prefixed `g_`
  ],
  "case_tags": [
    { "case_number": 12, "tag_id": "g_1" }
  ],
  "onboarding_completed": true
}
```

Notes:

- Cases are referenced by `case_number` (1–57) rather than the server's UUIDs, since the guest has no server account. This is the only schema deviation from the authenticated tables. The migration step (§6) translates `case_number` → `case_id` on import.
- Tag IDs are local-only (`g_<n>`) and remapped to server UUIDs during migration.
- The version field future-proofs the structure. v1 is what's described above.

---

## 5. Frontend behavior

### State management
Pinia stores expose the same shape regardless of mode:
- `authStore.isGuest` is `true` when the app is running off `localStorage`
- All other stores (`casesStore`, `studyStore`, `progressStore`, `tagsStore`) read/write through a thin adapter that dispatches to either the API or `localStorage` based on `authStore.isGuest`

### Persistence
- Every mutation through the adapter immediately persists the full `oll-guest-state` blob (debounced at ~250ms to coalesce rapid edits)
- On app load, if `oll-guest-state` exists and the user is unauthenticated, the splash routes to the dashboard in guest mode rather than `/login`. If both a guest blob and a valid auth cookie exist, the auth cookie wins and a banner offers to migrate the guest data (see §6)

### Banner
A persistent dismissible banner across guest sessions:

> "You're practicing as a guest. Progress is saved on this device only. **Save your progress** →"

Tapping the link routes to the `GuestUpgradeScreen`.

---

## 6. Migration: guest → account

Triggered when a guest taps the upgrade CTA and successfully completes registration via `GuestUpgradeScreen`.

### Flow
1. User submits `display_name`, `email`, `password`, reCAPTCHA token through the standard `POST /auth/register` endpoint
2. **Additionally**, the request body includes a `guest_state` field carrying the entire `localStorage` blob (or omitted if the user has no guest progress)
3. Backend verifies reCAPTCHA, creates the user, generates the verification code as normal
4. **If `guest_state` is present**, the backend imports it transactionally:
   - For each entry in `progress`, look up the matching `case_id` and insert into `user_case_progress`
   - For each entry in `settings`, insert into `user_case_settings` (translating `case_number` and `result_case_number` to UUIDs)
   - Create `tags` rows for each guest tag, keeping a temporary `g_<n>` → UUID map
   - Insert `case_tags` rows using the map
   - Set `users.streak_count` and `users.last_practice_date` from the guest blob
5. Backend issues the verification email as usual
6. Frontend clears `oll-guest-state` from `localStorage` on a 200 response
7. User is routed to the verify-email screen; everything else proceeds like a normal registration

### Failure handling
- Network failure: `oll-guest-state` is preserved (we only clear on confirmed 200). User can retry.
- Validation errors (e.g. malformed blob): backend returns 400 with detail; frontend surfaces it without clearing the blob.
- Partial progress collisions are not possible — the user is brand new, so their `user_case_progress` table is empty before import.

### Migration on existing-account sign-in
If a guest taps "Sign in" instead of "Create account":
- Standard login flow runs, no progress migrated
- Frontend keeps `oll-guest-state` around and shows a one-time banner: "We found guest progress on this device. **Merge it into this account?** / **Discard.**"
- If merge: a separate `POST /auth/merge-guest-state` (auth required) runs the same import logic, taking the higher of (server vs guest) `interval_days` per case to avoid overwriting better progress. Streak takes the max.
- If discard: clear `oll-guest-state`

---

## 7. Backend additions

### Endpoints
- `POST /auth/register` — already in the main spec; gains an optional `guest_state` field in the body
- `POST /auth/merge-guest-state` — auth required; accepts the same `guest_state` shape; merges with `MAX(interval_days)` and `MAX(streak)` rules

### Validation
- `guest_state.version` must equal `1`
- All `case_number` values must be in `[1, 57]`
- `progress[*].ease_factor ∈ [1.3, 5.0]`, `interval_days >= 0`, `repetitions >= 0`
- Tag names: max length 50, max 100 tags, max 1000 case-tag links

### Schema
No schema changes from the main spec. Guest data lives entirely client-side until migration.

---

## 8. Open questions for guest mode

These need decisions before guest-mode implementation begins (last MVP step), but are surfaced now so the build doesn't paint into a corner.

1. **Direct guest URL.** Should there be a `/guest` route that bypasses the splash + login entirely (useful for evaluating the app without a click)? Default proposal: no — the "Continue as guest" button on `/login` is enough.
2. **Banner persistence.** Once a guest dismisses the upgrade banner, does it come back the next session? Default proposal: yes, every time the app loads, but with a "Don't show this again until I have 10+ reviewed cases" dismiss option.
3. **Tag count caps.** Are the `100 tags / 1000 case-tag links` limits in §7 reasonable? Alternative: enforce the same caps for authenticated users.
4. **Server-side guest fingerprinting.** Should the upgrade endpoint rate-limit by IP or fingerprint to prevent abuse (creating many accounts from imported guest blobs)? Default proposal: standard per-IP rate limit on `/auth/register` covers it; no special treatment.
5. **localStorage size.** A guest with all 57 cases + dense overrides + many tags could approach 50–100 KB. Browsers allow ~5 MB; not a concern. Worth measuring once the renderer is in.
6. **Version migration policy.** When `version` jumps from 1 to 2, who runs the migration — frontend on load, or backend on merge? Default proposal: frontend on load, since the data is local. Backend rejects unknown versions.
7. **Sign-in over guest.** When a returning user signs in with an existing account on a device with guest data, the merge banner (§6) is the answer. But what about a guest who *has* an account on another device — should we surface "Sign in to sync" earlier in the flow? Default proposal: no, the existing footer link on the splash/login is enough.
