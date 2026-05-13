# Milestone 6 — Guest Mode

Detailed design + story list for M6 — the final MVP feature. Scope set in `docs/milestones/README.md`. Conceptual spec is `docs/guest_mode_design_doc.md`; this doc is the implementation plan, including conflicts the original guest spec hadn't anticipated (it predates M4's tag rework and M5's landing page).

---

## 1. Goal recap

By the end of M6:

- A new visitor can enter guest mode from either the public landing page or the login screen ("Continue as guest"). Inside guest mode the app behaves identically to the authed path — practice, free study, cases browser, progress, free-form tags, streak, onboarding stub.
- Guest data persists in `localStorage` under the single key `oll-guest-state` (schema v1). Every mutation is debounced-flushed at ~250 ms so rapid edits coalesce.
- A persistent dismissible banner — "You're practicing as a guest. Progress is saved on this device only. **Save your progress** →" — sits above the tab bar and links to the upgrade flow.
- `GuestUpgradeScreen` (a registration variant) sends the guest blob along with the standard register payload; the backend imports it transactionally on success and the frontend clears `oll-guest-state` on a 200.
- A returning user who signs into an existing account on a device that already has guest data sees a one-time merge banner offering to fold the guest data into the account (max-of-server-vs-guest per case, max streak) — or discard.
- The M5 onboarding stub fires once on first guest-mode entry (`OnboardingView` reused) and never again on this device.
- All §10 mobile + accessibility passes hold.

Out of scope (deferred):
- Direct `/guest` URL bypassing landing/login (Q1 — answered "no").
- Server-side guest fingerprinting beyond the existing per-IP register limit (Q4).
- Post-MVP feature parity (PLL/F2L, dark mode, etc.) — same exclusions as the rest of the spec.

---

## 2. Architecture for M6

Guest mode is a parallel persistence backend, not a parallel app. The Pinia stores expose the same shape regardless of mode; an adapter layer swaps the data source based on `authStore.status`.

### Backend additions
- **`POST /auth/register`** gains an optional `guest_state` field on the request body. When present and the user is created successfully, the backend imports the blob in the same transaction.
- **`POST /auth/merge-guest-state`** — auth-gated, accepts the same `guest_state` shape. Per-case merge rule: keep the row with the higher `interval_days`; ties go to the server. Streak takes the max.
- **`GET /cases` and `GET /cases/:id` lose the auth gate.** Both already return the global case set when there are no overrides; making them anonymous-capable lets guests browse the same dataset without bundling a 57-case fixture in the frontend. The override merge step only runs when `AuthUser` is present (extractor becomes `Option<AuthUser>`).
- **Tag-cap validation** added to the existing `PATCH /cases/:id/settings`: max 100 distinct tags per user, max 50 chars per tag, max 1000 total case-tag links per user. Same caps applied to the guest-state import path. (Q3 — applied to authed users for consistency.)

### Frontend additions
- **localStorage adapter** under `@/lib/guest/`: `storage.ts` (load/save/debounce), `state.ts` (typed schema v1), `migrate.ts` (forward-version migrations).
- **SM-2 ported to TypeScript** (`@/lib/sm2.ts`) so guest reviews don't need a server roundtrip. Rust impl in `backend/src/srs/` stays canonical; TS port mirrors its constants and tests.
- **Pinia adapter pattern** for the four user-scoped stores (`cases`, `study`, `progress`, plus a new `tags` if needed). Each store gets a small "driver" indirection — API driver vs localStorage driver — selected on `authStore.status`. `$reset` semantics extend: switching modes flushes both drivers' caches.
- **`AuthStatus` rename.** Current `'guest'` (= "unauthenticated visitor") becomes `'anon'`; new `'guest'` means "running off localStorage with a guest blob loaded." Bootstrap logic: with neither cookie nor blob → `'anon'`; with valid cookie → `'authed'`; with blob and no cookie → `'guest'`; with both → `'authed'` and the merge banner offers to fold the blob in (per Q7 / §6 of the original spec).
- **`<GuestBanner>`** — fixed above the tab bar inside `AppShell`, only renders when `status === 'guest'`. Dismissible per session; reappears on next bootstrap unless the user picked "Don't show again until I have 10+ reviewed cases" (Q2). Dismissal state lives on the guest blob itself (`banner_dismissed_at` + `banner_suppressed_until_reviews`).
- **`<GuestUpgradeScreen>`** at `/upgrade` — same form as `RegisterView` but bundles the current guest blob into the request body. Discoverable via the banner CTA and a "Save your progress" link in `SettingsView` while in guest mode.
- **Sign-in-with-guest-data merge.** After a successful login on a device with a guest blob, route to a one-time merge prompt (`<GuestMergePrompt>`). Two outcomes: merge (POST `/auth/merge-guest-state`, then clear blob) or discard (clear blob).
- **Entry surfaces.** On `LandingView`, the hero's primary "Sign in →" CTA is **replaced** by "Continue as guest" (Q-B clarified). The top-right "Sign in" link and the bottom-of-page closing "Sign in →" CTA stay as the existing-user paths. On `LoginView`, "Continue as guest" lands as a footer link.
- **Onboarding for guests.** Same `OnboardingView`. Triggered once when the guest blob is created and `onboarding_completed === false`. Final step writes `onboarding_completed: true` to the blob (no backend call).

### What's already done (validated during M6 survey)
- `users.streak_count` + `users.last_practice_date` columns exist (`backend/src/study/mod.rs:21,155`). Import path can write to them directly.
- `tags TEXT[]` schema lands on both `cases` and `user_case_settings` from M4. The localStorage shape uses the same per-case array (Q-A — drop the doc's separate `tags`/`case_tags` arrays).
- `OnboardingView` is generic over auth state; reusing it is a one-line trigger change.
- Splash min-hold + bootstrap promise pattern (`stores/auth.ts:74`) extends naturally to the blob check.

---

## 3. Schema — M6 changes

No schema migrations. Behavior change only: `PATCH /cases/:id/settings` and the new import endpoints enforce tag caps.

A future migration could add an enforce-at-DB-level constraint (a `CHECK (cardinality(tags) <= 100)`) but that's deferred — server-side validation is sufficient.

---

## 4. API surface — M6 additions and changes

Prefix `/api/v1`.

### Changed

| Method | Endpoint | Change |
|--------|----------|--------|
| GET | `/cases` | Auth optional. Anonymous → return global cases; authed → merge user overrides as today. |
| GET | `/cases/:id` | Same — auth optional. |
| PATCH | `/cases/:id/settings` | Adds tag-cap validation: max 100 distinct tags/user, max 50 chars/tag, max 1000 links/user. |
| POST | `/auth/register` | Body gains optional `guest_state` field. When present, validated + imported in the same transaction as user creation. |

### New

| Method | Endpoint | Body | Returns |
|--------|----------|------|---------|
| POST | `/auth/merge-guest-state` | `{ guest_state: GuestState }` | `{ ok: true, merged: { cases: <count>, tags: <count> } }` |

`GuestState` JSON shape (matches §5 below). Validation:

- `version` must equal `1`.
- All `case_number` values in `[1, 57]`.
- Each `progress` entry: `ease_factor ∈ [1.3, 5.0]`, `interval_days >= 0`, `repetitions >= 0`, `due_date` parseable.
- Tag names: trimmed, non-empty, ≤ 50 chars, deduped after normalization. Cap counts as in §2.
- Whole blob bounded at 256 KB (defensive — actual blobs run ≤ 50 KB).

Merge rules on `/auth/merge-guest-state`:
- For each guest progress entry, compare against the existing server row by `case_id`. Keep whichever has the higher `interval_days`. Ties → server.
- For each guest setting, only insert when no server override exists. Don't overwrite explicit user choices.
- Streak: `MAX(server, guest)` for `streak_count`; `last_practice_date` is the later of the two.
- Tags: union, normalized + deduped, capped at 100.

---

## 5. Frontend — M6

### `localStorage` schema v1

```jsonc
{
  "version": 1,
  "display_name": null,
  "created_at": "2026-05-12T19:42:00Z",
  "streak_count": 0,
  "last_practice_date": null,
  "onboarding_completed": false,
  "banner_dismissed_at": null,
  "banner_suppressed_until_reviews": null,
  "settings": {
    // keyed by case_number (1–57); only entries with at least one override present
    "12": {
      "nickname": "Slash",
      "algorithm": "F R U R' U' F'",
      "result_case_number": 21,
      "result_rotation": 1,
      "tags": ["needs work", "fish"]
    }
  },
  "progress": {
    // keyed by case_number; only entries with at least one review
    "12": {
      "ease_factor": 2.5,
      "interval_days": 6,
      "repetitions": 2,
      "due_date": "2026-05-18",
      "last_grade": 2,
      "last_reviewed": "2026-05-12T19:41:23Z"
    }
  }
}
```

Differences from the original `guest_mode_design_doc.md` §4:
- `tags` is per-case `string[]` inside each settings entry (matches M4 schema). Standalone `tags[]` and `case_tags[]` arrays are dropped (Q-A).
- Adds `banner_dismissed_at` + `banner_suppressed_until_reviews` to support Q2's "don't show again" path.
- `onboarding_completed` retained — semantics identical to authed `users.has_seen_onboarding`.
- `tier2_tag` removed (the M4 collapse is reflected — guests use the same flat tag list).

### Routes

| Path | Auth | Notes |
|------|:---:|------|
| `/upgrade` | guest only | New `<GuestUpgradeScreen>`. Routes to verify-email on success. |

Existing routes are unchanged. The router guard learns the new status:
- `requiresAuth` routes admit `'authed'` but not `'guest'` or `'anon'`.
- `'guest'` users hitting `/login`, `/register`, etc. (currently `guestOnly`) — still admitted; the language was never "anon-only", just "not-yet-signed-in." Keep `guestOnly` as a route flag but add a derived store getter `isUnauthed = status !== 'authed'` for clarity. `LandingView`'s redirect-when-authed rule extends: `'guest'` users hitting `/` route to `/practice` (their guest dashboard).

### Components / views

- **`<GuestBanner>`** — fixed-position bar inside `AppShell`, above `<TabBar>`. Wordmark-on-paper aesthetic; copy + CTA per §1; small × dismiss. Dismiss writes `banner_dismissed_at = now`. If user opts into "Don't show again until 10+ reviews," writes `banner_suppressed_until_reviews = 10` and the banner stays hidden until `Object.keys(progress).length >= 10`.
- **`<GuestUpgradeScreen>`** at `/upgrade` — clones `RegisterView` form fields (display name, email, password, confirm, recaptcha) and a new lede paragraph: "Create an account to keep your progress safe across devices." On submit, calls `auth.upgradeFromGuest(payload)` which posts the standard register body plus `guest_state`. On 200: clear blob, route to `/verify-email`.
- **`<GuestMergePrompt>`** — modal-card overlay shown once after a sign-in lands on a device with a guest blob. Two buttons: "Merge into this account" (posts `/auth/merge-guest-state`, then clears blob) and "Discard guest data" (clears blob, no API call). After either, the prompt never renders again on this device.

### Pinia adapter pattern

Each user-scoped store gets a `driver` indirection:

```ts
interface CasesDriver {
  list(): Promise<Case[]>
  detail(id: string): Promise<Case>
  updateSettings(id: string, patch: SettingsPatch): Promise<Case>
}

const driver: CasesDriver =
  authStore.status === 'guest' ? guestCasesDriver : apiCasesDriver
```

The API driver is what already exists. The guest driver:
- Reads cases from a public `GET /cases` (anonymous) and merges localStorage settings on top per request — caches the global list for the session, hits `localStorage` for overrides.
- Writes settings to localStorage; nothing leaves the device.

Same pattern for `study`, `progress`. `study` adapter calls `@/lib/sm2.ts` for the schedule update instead of POST `/study/:id/review`.

### Bootstrap flow (extended)

```
splash → bootstrap()
  read localStorage 'oll-guest-state'
  GET /auth/me  (existing)

  if /auth/me 200:
    status = 'authed'
    if guest blob present:
      schedule <GuestMergePrompt> on next dashboard render
  else if guest blob present:
    status = 'guest'
    if blob.onboarding_completed === false → route to /welcome
  else:
    status = 'anon'
```

`resetUserScopedStores()` (already in `auth.ts`) extends to also clear the in-memory copy of the guest blob when leaving guest mode.

---

## 6. Security notes specific to M6

- **Trust no client-supplied IDs.** The blob references cases by `case_number` (1–57), which the backend translates to `case_id` UUIDs from the seed. No client UUIDs accepted; tag IDs were never stored client-side in v1.
- **Validate everything on import.** Bounds + lengths + cardinality. A malformed blob returns 400 with detail; the blob stays on-device so the user can retry.
- **`/auth/merge-guest-state` is auth-gated.** The endpoint only ever writes to the calling user's row.
- **Rate limiting.** `/auth/register` already rate-limits per-IP. `/auth/merge-guest-state` reuses the same per-IP limiter under a different key.
- **Public `GET /cases` returns global data only.** No user content (no overrides, no progress, nothing per-account). The `Option<AuthUser>` extractor must default to "no overrides" — never silently return another user's data.
- **localStorage isolation.** `oll-guest-state` is per-origin; SameSite cookies are unaffected. Clearing site data wipes guest progress with no recovery (documented to the user via the upgrade banner copy).

---

## 7. Testing strategy

**Backend (cargo test):**
- `routes::cases::list` returns globals when `Option<AuthUser>` is `None`.
- `routes::cases::list` returns merged user data when authenticated (regression).
- Tag-cap validation: writing 101 distinct tags rejects; 51-char tag rejects; 1001-link total rejects.
- `/auth/register` with `guest_state` creates user + imports progress + settings + tags transactionally; rollback on import failure.
- `/auth/register` with malformed `guest_state` returns 400 and does not create the user.
- `/auth/merge-guest-state`: keeps higher `interval_days`; doesn't overwrite explicit settings; max-streak; tag union deduped; auth-gate rejects unauthed.
- Bounds + version + size validation reject crafted payloads.

**Frontend unit (Vitest):**
- `@/lib/guest/storage.ts` debounced writer coalesces rapid mutations.
- `@/lib/sm2.ts` matches the Rust SM-2 reference (port a couple of the existing Rust test cases as parity tests).
- Adapter selection: `cases.list` round-trips through the API driver in `'authed'` mode and through the localStorage driver in `'guest'` mode.
- `GuestBanner` "don't show until N reviews" path: hide while `Object.keys(progress).length < 10`, reappear at `>= 10`.

**Manual QA:** §10 below.

---

## 8. Configuration / environment

No new env vars. Storage key constant lives in `@/lib/guest/storage.ts`.

---

## 9. Migration / data risk

No schema migrations. Two behavior changes for existing authed users:

- **Tag-cap validation** newly rejects writes that exceed 100 tags / 50 chars / 1000 links. Inspection: current schema usage tops out at single-digit tags per case; no existing user is anywhere near the caps.
- **`GET /cases` un-authed.** Previously returned 401 to anonymous callers; now returns the global case list. No user data leaks because the override merge step only runs when `Option<AuthUser>` is `Some`.

---

## 10. "Done when" checklist (run on the deployed instance)

- [ ] Visit `/` while signed out → hero shows a "Continue as guest" primary CTA in place of the previous "Sign in →" button. Top-right "Sign in" link and bottom closing-section "Sign in →" still visible.
- [ ] Tap "Continue as guest" → routes to `/welcome` (onboarding stub, step 1).
- [ ] Complete onboarding → lands on `/practice`. Banner visible above tab bar.
- [ ] Browse to `/cases` → all 57 cases render. Open one → algorithm + result diagram render.
- [ ] Edit a case (nickname / algorithm / result mapping / tags) → reload the page → edit persists.
- [ ] Add a free-form tag in case detail → tag chip appears in cases browser filter row.
- [ ] Try to add a 101st tag → form rejects with "Too many tags."
- [ ] Run a study session — pattern → reveal → grade → SM-2 schedules the case → next card. Reload mid-session → safe-bounce back to `/practice`.
- [ ] Grade across multiple days (or simulate by clearing `last_practice_date`) → streak ticks correctly per the day-rollover rule.
- [ ] Free study → all four filter axes work as in authed mode.
- [ ] Progress page → state distribution + per-case list reflect localStorage.
- [ ] Banner dismiss × → reappears next session.
- [ ] Banner dismiss with "Don't show until 10+ reviews" → hidden until `progress.size >= 10`, then reappears.
- [ ] Tap "Save your progress →" on the banner → routes to `/upgrade`.
- [ ] Submit upgrade form → server creates account + imports data + clears `oll-guest-state`. Verify-email lands.
- [ ] Verify the email → onboarding does *not* re-fire (already completed in guest mode; backend `has_seen_onboarding` flipped on import or via the post-verify trigger). Practice tab shows the imported progress.
- [ ] On a *different* browser, sign into the upgraded account → progress matches.
- [ ] Sign out, enter guest mode again on the same browser → blob is empty (cleared on upgrade). Practice from scratch.
- [ ] Build a fresh guest blob with some progress, then sign into a *different* existing account → merge prompt renders. "Merge into this account" → progress folds in (max-rule). "Discard" on a different test → blob cleared, no merge.
- [ ] Settings link "Save your progress" appears in guest mode and routes to `/upgrade`. Hidden in authed mode.
- [ ] Sign-in-then-merge: server has interval=10 for case 5, guest has interval=2 → after merge, server keeps 10. Server has interval=2, guest has interval=10 → after merge, server takes 10.
- [ ] Mobile QA — iOS Safari + Android Chrome: banner renders without overlapping tab bar; upgrade form usable on a small screen.
- [ ] `localStorage` size after dense use stays < 100 KB (DevTools).
- [ ] Catch-all 404, focus rings, label/aria coverage from M5 hold for the new screens.

---

## 11. Story list

Pairs backend + frontend per the project's principle.

### Public read endpoints + tag caps (backend)
- [ ] **B1.** Make `Option<AuthUser>` the extractor on `GET /cases` and `GET /cases/:id`. Anonymous → globals; authed → merged. Regression tests for both paths.
- [ ] **B2.** Tag-cap validation on `PATCH /cases/:id/settings`. Tests for over-cap rejections + boundary acceptances.

### Guest-state import (backend)
- [ ] **B3.** Extend `POST /auth/register` to accept optional `guest_state`. Validate; import transactionally; tests for happy path, malformed blob, mid-import rollback.
- [ ] **B4.** New `POST /auth/merge-guest-state`. Auth-gated; max-rule merge; tests cover all merge edges (interval tie, settings collision, streak max, tag union dedup).

### SM-2 port + storage layer (frontend)
- [ ] **D1.** `@/lib/sm2.ts` — port the Rust algorithm + constants. Vitest parity cases mirroring the Rust tests.
- [ ] **D2.** `@/lib/guest/storage.ts` — load/save/debounce, schema v1 typing, `useGuestState()` composable.
- [ ] **D3.** `AuthStatus` rename `'guest'` → `'anon'` and introduce new `'guest'` semantics. Update `auth.ts` bootstrap, router guards, every consumer of `status`. Add `isAuthed` / `isGuest` / `isAnon` derived getters.

### Adapter pattern (frontend)
- [ ] **D4.** Cases driver split — `apiCasesDriver` (existing) + `guestCasesDriver`. Store selects on `status`. Existing tests stay green; new tests cover guest reads + writes.
- [ ] **D5.** Study driver split — `guestStudyDriver` runs SM-2 client-side and writes to localStorage. Streak ticks per the same day-rollover rule.
- [ ] **D6.** Progress driver split — derives the state distribution and per-case list from localStorage in guest mode.

### Entry surfaces + banner (frontend)
- [ ] **D7.** Hero CTA swap on `LandingView` — replace primary "Sign in →" with "Continue as guest"; top-right + closing "Sign in →" stay. Add "Continue as guest" footer link on `LoginView`. Both routes go through `auth.startGuestMode()` which creates the blob, sets status, fires the onboarding trigger if first-time.
- [ ] **D8.** `<GuestBanner>` inside `AppShell`, only visible when `status === 'guest'`. Dismiss × + dropdown for "Don't show until 10+ reviews."

### Upgrade + merge flows (frontend)
- [ ] **D9.** `<GuestUpgradeScreen>` at `/upgrade`. Clones the register form; on submit calls `auth.upgradeFromGuest(payload)` which posts `{ ...register, guest_state }` and clears the blob on 200.
- [ ] **D10.** `<GuestMergePrompt>` post-login on a device with a guest blob. Merge / discard branches; blob cleared on either.
- [ ] **D11.** Settings: "Save your progress" link visible in guest mode, hidden in authed mode.

### Polish
- [ ] **D12.** Onboarding gate update — fire `OnboardingView` on first guest-mode entry; final step writes `onboarding_completed: true` to the blob (no API call).

### QA
- [ ] **E1.** Walk §10 on the deployed instance, including mobile + the cross-browser merge scenarios.

---

## 12. Notes / open items

These need confirmation before B1 lands. Each carries my proposed answer; push back on any.

1. **Direct `/guest` URL bypassing landing/login.** **Proposed: no.** The "Continue as guest" link on landing + login is enough; an extra route is more surface area than it's worth.
Sounds good

2. **Banner persistence after dismissal.** **Proposed: returns every session, with a "Don't show again until I have 10+ reviewed cases" option on the dismiss dropdown.** The threshold (10) is arbitrary but matches the doc's example; happy to tune.
that's worth a shot

3. **Tag count caps.** **Proposed: enforce `100 tags / 50 chars / 1000 links` for both guests *and* authed users.** Same code path, single mental model. Authed users today are nowhere near these limits.
ok

4. **Server-side guest fingerprinting.** **Proposed: no.** The standard per-IP rate limit on `/auth/register` covers the abuse vector. `/auth/merge-guest-state` reuses the same limiter.
sounds good

5. **localStorage size.** Non-issue at v1 (~50 KB worst case vs ~5 MB browser limit). Surface in QA if it ever gets close.
ok

6. **Version migration policy.** **Proposed: frontend on load.** Backend rejects unknown versions on merge and import. v1 → v2 (when it happens) runs client-side before any read.
ok

7. **"Sign in to sync" surfacing.** **Proposed: no extra surfacing.** The existing footer link on landing + login is sufficient; a guest who already has an account can find it. The merge banner handles the after-the-fact path.
sonds good

### Conflicts created by M4 + M5

A. **Tag schema in the blob.** M4 collapsed `tier2_tag` + junction tables into `tags TEXT[]`. **Proposed: blob's `settings[case_number].tags` is `string[]`; drop the standalone `tags[]` and `case_tags[]` arrays.** The schema in §5 reflects this.
okay

B. **"Continue as guest" placement.** **Proposed: both LandingView (below hero CTAs) and LoginView (footer link).** Landing is the primary first-impression surface; login catches deep-linkers. One shared `auth.startGuestMode()` action behind both buttons.
Let it replace the sign in at the hero section (keep sign it at bottom and top right corner)

C. **`AuthStatus` naming collision.** Current `'guest'` means "unauthenticated." **Proposed: rename to `'anon'`; introduce new `'guest'` for guest-mode-with-localStorage.** The other option (separate `isGuestMode` flag alongside the existing status) muddies the guard logic; prefer the rename.
Looks good

D. **Onboarding for guests.** **Proposed: same `OnboardingView`. Triggered once on first guest-mode entry. Final step writes `onboarding_completed: true` to the blob (no backend roundtrip — there is no backend for this user yet).** When a guest later upgrades, the import path sets `users.has_seen_onboarding = TRUE` from the blob, so re-verification on a new device skips onboarding.
sounds good

---

## 13. Looking back

A retrospective pass added after the milestone shipped. Fill in honestly —
the value of these notes is the friction they capture, not a victory lap.

- **What I'd do differently if I started this milestone today:**

- **Surprises during execution:**

- **Decisions that turned out to matter more than expected:**

- **Decisions that turned out not to matter:**

- **What this milestone taught me that I'd carry into future work:**
