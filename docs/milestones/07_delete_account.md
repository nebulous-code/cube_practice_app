# Milestone 7 — Delete Account

Scope: a logged-in user can permanently delete their account and all associated data from Settings. Required pre-launch for legal compliance (GDPR Article 17, CCPA right-to-delete) and standard app-store / web-app expectation. Tight milestone — one new endpoint, one Settings section, one confirm flow.

---

## 1. Goal recap

By the end of M7:

- A "Delete account" section is reachable from `SettingsView` for authed users (hidden in guest mode — guests have no account; their data clears with `localStorage`).
- Deletion is two-step + password-gated to make accidental clicks expensive: tap → expand confirm pane with prominent warning copy, password field, and a red "Delete forever" button.
- `DELETE /auth/me` runs server-side: revokes every session, deletes the `users` row, lets the existing `ON DELETE CASCADE` foreign keys clean up `sessions`, `user_case_settings`, and `user_case_progress`. Clears the session cookie on the response. No grace period — the action is final.
- Frontend post-delete: clears all in-memory user state, routes to `/login` with a small "Account deleted." note, the user lands as `'anon'`.
- Tests cover the SQL cascade (rows for the deleted user disappear; other users untouched), wrong-password rejection, idempotent unauth on a deleted session, and the route-handler contract.

Out of scope (deferred):
- **Soft delete / 30-day recovery window.** Hard delete is simpler and matches the schema's CASCADE design. Add later if support load demands it.
- **Data export ("Download my data") before deletion.** Real value but post-MVP — most users want delete-without-keepsake. Tracked for post-MVP.
- **Email confirmation step** (deletion link mailed to the user). Password-gating is sufficient at this layer; email confirmation is friction without much added safety once the user is already authenticated. Revisit if abuse patterns emerge.
- **Audit logging** of deletions. We log via tracing today; structured deletion records for compliance reporting are post-MVP.
- **Guest "Discard guest data" Settings entry.** Trivially `localStorage.removeItem` — separate ticket if/when surfaced.

---

## 2. Architecture for M7

Backend is the heavy side. Frontend is essentially a confirmation pane.

### Backend additions
- **`DELETE /auth/me`** — auth-gated. Body: `{ current_password: string }`. Verifies the password against the stored hash, runs a single `DELETE FROM users WHERE id = $1` (CASCADE handles the rest), clears the session cookie. Returns `{ ok: true }` on success.
- **Rate limit reuse.** Per-user key on the rate limiter: `delete:user:<uuid>` with the same window as `sign-out-all`. Defense in depth — primary gate is the password check.
- No new module needed; the handler lives in `routes/auth.rs` next to the other `/auth/me` endpoints.

### Frontend additions
- **Settings → Delete account card.** Visible only when `auth.isAuthed`; below the existing Sessions card. Two-step:
  1. Initial state: a single ghost-style "Delete account" button labeled with red ink-faint and a one-line warning paragraph.
  2. Expanded state (after click): warning copy explaining "This is permanent — every review, override, and session goes away," a password field, "Cancel" + "Delete forever" actions. The destructive button stays disabled until the password field is non-empty.
- **`auth.deleteAccount(currentPassword)` action.** POSTs (DELETE) to `/auth/me`, on 200 clears `user`, sets `status = 'anon'`, calls `resetUserScopedStores()`, clears any guest blob (defense in depth). Throws on 401 / validation so the form can show the error inline.
- **Post-delete routing.** Settings's delete handler routes to `/login?deleted=1`; LoginView reads the query param and shows a one-time "Account deleted." note above the form. The note clears on first interaction.

### Reuse from M1–M6
- Existing `auth.changePassword` / `auth.signOutAll` flows are the closest precedent for "password-confirmed destructive action." Same error-shape conventions, same field-style validation.
- `ON DELETE CASCADE` from M1 (sessions) and M2 (user_case_settings, user_case_progress) does all the cleanup. No schema change.
- `clear_session_cookie()` from M1 hands back the cookie wipe for the response.

---

## 3. Schema — M7 changes

None. The cascade behavior already exists:

- `sessions.user_id REFERENCES users(id) ON DELETE CASCADE` (migration 0001)
- `user_case_settings.user_id REFERENCES users(id) ON DELETE CASCADE` (migration 0002)
- `user_case_progress.user_id REFERENCES users(id) ON DELETE CASCADE` (migration 0004)

A future migration could add a `deleted_at TIMESTAMPTZ` for soft-delete; out of scope here.

---

## 4. API surface — M7 additions

Prefix `/api/v1`. Auth-gated.

### New

| Method | Endpoint | Body | Returns |
|--------|----------|------|---------|
| DELETE | `/auth/me` | `{ current_password: string }` | `{ ok: true }` + session cookie cleared |

Errors:
- `401 invalid_password` when the supplied password doesn't match.
- `401 unauthorized` when the cookie is missing/expired (standard auth gate).
- `429 rate_limited` (defense-in-depth limiter — see §6).

The endpoint is **not** idempotent in the strict sense: a second call returns 401 because the session is gone. That's the expected user experience.

### No changes to existing endpoints

`POST /auth/sign-out-all` stays — it's a different action (revoke sessions, keep account). The Settings UI keeps both side by side.

---

## 5. Frontend — M7

### Routes

No new routes. The `/login` view gains a query-param read (`?deleted=1`) for the post-delete note.

### Components / views

- **`SettingsView` — Delete account card.** Two-step pattern matching the existing "Sign out everywhere" expand. Layout:

  ```
  [Card]
   Eyebrow: "Delete account"
   Body:    "Permanently remove your account and all data."
   [Button: "Delete account"]   ← initial state, ghost / red ink-faint

  → Expanded:
   Warning paragraph: "This can't be undone. Every review, every
                       override, every session — gone."
   [Password field] (label: "Current password")
   [Inline error]
   [Cancel] [Delete forever]    ← destructive primary button
  ```

  Visible only when `auth.isAuthed`. Hidden in guest mode (different exit path — Discard guest data is a separate ticket, see §1 out-of-scope).

- **`auth.deleteAccount(currentPassword)` action.** Mirrors the shape of `signOutAll`. On success clears `user`, `status = 'anon'`, `pendingMergePrompt = null`, calls `resetUserScopedStores()`, calls `clearGuestState()` (cheap noop if no blob), returns void. Throws on failure.

- **`LoginView` post-delete note.** Reads `route.query.deleted === '1'` on mount and renders a small italic line above the form: "Account deleted. You're signed out." The note clears once the user types in the email field or after a successful sign-in.

### Removed surfaces

None.

### What's already done

- Password-confirmed destructive flow lives in `SettingsView.vue`'s "Sign out everywhere" handler and `RegisterView.vue`'s validation pattern. M7 follows the same idiom.
- `clear_session_cookie()` exists in `backend/src/auth/cookie.rs`.

---

## 6. Security notes specific to M7

- **Password gate is the primary control.** A logged-in attacker with cookie access (XSS, browser session theft) cannot delete the account without also knowing the password. Same model as `sign-out-all`.
- **No row-level surprises.** The cascade is declarative — every FK pointing at `users.id` already has `ON DELETE CASCADE`. The §3 schema check is the audit; if a future migration adds another FK without CASCADE, the delete would fail loudly rather than orphan rows.
- **Rate limit.** Per-user limiter at `delete:user:<uuid>` — 3 attempts per hour. Stops a stuck-cookie attacker from grinding through password guesses. The route returns the same 401 shape for "wrong password" and "limited" so the limiter doesn't help an attacker enumerate state.
- **Cookie wipe on success.** `clear_session_cookie()` is added to the response jar even though the underlying session row is gone — keeps the browser from holding stale state.
- **Logging.** `tracing::info!(user_id = %id, "account deleted")` at the route level for operational visibility. We don't retain PII (the `id` is the UUID and the row is gone).
- **No privilege escalation across users.** The DELETE only ever runs against the calling user's `id` (extracted from `AuthUser`). There is no path that takes a user_id from the request body.

---

## 7. Testing strategy

**Backend (cargo test):**
- Happy path: register → login → call `delete_account` with correct password → users row gone, sessions for that user gone, user_case_settings + user_case_progress for that user gone.
- Cross-user isolation: alice deletes; bob's rows untouched.
- Wrong password: returns 401 (`invalid_password`); user row still present; sessions still active.
- Cascade audit: assert every FK pointing at `users.id` has CASCADE — schema introspection test so a future migration that drops the cascade fails CI.
- Cookie clear assertion (handler-level): response cookie jar contains the wipe directive.

**Frontend unit (Vitest):**
- `auth.deleteAccount` action: stub api.delete, verify it clears user/status/stores on 200, throws on 401 without clearing.
- LoginView `?deleted=1` query param: renders the note; clears on email keystroke.

**Manual QA:** §10 below.

---

## 8. Configuration / environment

No new env vars. The rate-limiter window for `delete:user:<uuid>` is a code constant.

---

## 9. Migration / data risk

No migration. Behavior risk:

- **Accidental deletion** is the main concern. Mitigations: password gate, two-step expand, red destructive button styling, explicit "This can't be undone" warning copy.
- **Cascade audit** in tests guards against a future migration silently dropping cleanup behavior.
- **No grace period.** Recoverable accidents are a support burden we're not equipped for pre-launch. Document the finality clearly. Revisit post-launch if the support pattern justifies a soft-delete column.

---

## 10. "Done when" checklist (run on the deployed instance)

A literal QA script. M7 closes when every line passes.

- [ ] Settings shows "Delete account" card for an authed user. Hidden in guest mode.
- [ ] Initial click expands the confirm pane; warning copy and password field render.
- [ ] "Delete forever" stays disabled until the password field is non-empty.
- [ ] Wrong password → inline error, account untouched, can re-enter.
- [ ] Correct password → response 200, browser routes to `/login?deleted=1`, the "Account deleted." note shows.
- [ ] Note clears once the user types in the email field on `/login`.
- [ ] Re-login attempt with the deleted credentials → standard "Invalid credentials" flow.
- [ ] DB inspection (or `/auth/me` curl with the old cookie) → 401 — session is gone.
- [ ] Other test user's rows untouched (manual: spot-check a second account's `user_case_progress`).
- [ ] Mobile QA — iOS Safari + Android Chrome: confirm pane fits without horizontal scroll, destructive button readable, password field opens the right keyboard.
- [ ] Rate limit smoke: 4 wrong passwords in a row → 4th attempt rate-limited (returns 429).

---

## 11. Story list

Backend + frontend pairs.

### Backend
- [ ] **B1.** `DELETE /auth/me` handler in `routes/auth.rs`. Body deserializes `{ current_password }`; verifies via `verify_password`; runs `DELETE FROM users WHERE id = $1`; appends `clear_session_cookie()`. Adds rate-limit key + reuses the existing limiter. Tests cover happy path, cross-user isolation, wrong password, and the cookie-wipe contract.
- [ ] **B2.** Cascade audit test — `tests/users_cascade_schema.rs` queries `information_schema.referential_constraints` for every FK referencing `users.id` and asserts `delete_rule = 'CASCADE'`. Catches future migrations that drop the cascade.

### Frontend
- [ ] **D1.** `auth.deleteAccount(currentPassword)` Pinia action — DELETE `/auth/me`, on 200 clear local state + reset stores + clear guest blob. Throws on failure. Vitest covers the success and 401 paths against a stubbed API.
- [ ] **D2.** SettingsView Delete account card — two-step expand mirroring the Sign-out-everywhere pattern. Hidden when `auth.isGuest`. Inline error for wrong password.
- [ ] **D3.** LoginView post-delete note — read `?deleted=1` on mount, render note, clear on first email keystroke.

### QA
- [ ] **E1.** Walk §10 on the deployed instance.

---

## 12. Notes / open items

Each item has my proposed answer; push back on any.

1. **Hard delete vs soft delete.** **Proposed: hard delete.** Schema's already shaped for it (CASCADE everywhere). Soft delete adds a `deleted_at` column, scrub-PII migration, and ongoing complexity for marginal value pre-launch.

2. **Grace period (e.g. 30 days to undo).** **Proposed: none.** Pairs with the soft-delete decision. We make the action loud and password-gated; we don't make it reversible.

3. **Email confirmation before delete (in addition to password).** **Proposed: no.** The user is already authenticated; the password is fresh evidence of identity. An email round-trip adds friction without a clear threat model upgrade.

4. **Data export before delete.** **Proposed: defer.** Real value but post-MVP. The doc tracks it as out-of-scope for M7. Once we ship, watch for explicit user requests; a simple JSON dump endpoint is easy to add.

5. **Audit log table for deletions.** **Proposed: tracing log line is enough.** Structured compliance reporting is post-MVP. The `tracing::info!` line in the route handler is sufficient operational signal.

6. **Guest "Discard guest data" entry in Settings.** **Proposed: separate ticket.** Trivially `clearGuestState()`, but the UX (confirmation pane? warning copy?) deserves its own thinking. Out of scope for M7 since guest data isn't an "account" in the database sense.

7. **Rate-limit window.** **Proposed: 3 attempts per hour per user.** Same shape as `forgot-password`'s per-email window. Stops cookie-stealing brute-force, doesn't get in the way of legitimate users who fat-finger their password once.
