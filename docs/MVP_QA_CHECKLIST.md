# MVP QA Checklist

A single literal pass before public launch, against the deployed instance at
`https://cube.nebulouscode.com` hitting `https://api.cube.nebulouscode.com`.

This consolidates the per-milestone "Done when" checklists (M1–M7) and adds
coverage for the post-spec polish features that landed after M6 — pattern
shrink on reveal, repeat session, edit-from-reveal, the only/any-of toggles,
state filter chip on cases, and so on. Items that no longer apply (grouped-by-
Tier-2 layout from M2, "Browse cases" empty CTA, bottom-anchored guest banner)
have been pruned.

Walk top-to-bottom. Use two real devices for the cross-device checks (a
laptop + an iPhone or Android works). Anything that fails goes in
`docs/TODO.md` if the fix slips past launch.

---

## 1. Auth & accounts

### Registration
- [ ] `/register` rejects: invalid email format, password < 8 chars, missing display name, duplicate email
- [ ] Successful registration sends a verification email within ~30 seconds; email contains a 6-digit code
- [ ] Registration footer renders Terms + Privacy links; clicking either opens the static page and back-navigation preserves form input
- [ ] reCAPTCHA: registration with a tampered/missing token → `recaptcha_failed`
- [ ] Rate limit: 11th register from same IP within an hour → 429

### Verify email
- [ ] `/verify-email` accepts a valid 6-digit code; wrong code → `invalid_code`; expired code → `code_expired`
- [ ] After verify, user is logged in and lands on `/welcome` (first-run only)
- [ ] "Resend code" sends a fresh code; rapid-fire resends are 429'd after 1 in 60 s

### Login / logout / sessions
- [ ] `/login` rejects wrong password; unverified account routes back to verify with a fresh code
- [ ] Successful login routes to `?next=` when set, else `/practice`
- [ ] Refresh after login → still logged in
- [ ] Sign out → `/login`; refresh → still on `/login`
- [ ] Direct nav to `/settings` while signed out → redirects to `/login?next=/settings`
- [ ] Direct nav to `/login` while signed in → redirects to `/practice`
- [ ] Rate limit: 21st login from same IP within a minute → 429

### Forgot / reset password
- [ ] `/forgot-password` sends a reset code; reset code works exactly once; expired reset code is rejected
- [ ] After password reset, sessions on a second browser/device are revoked (refresh → `/login`)

### Settings → Account
- [ ] Change display name → reflected in header / avatar
- [ ] Change email → banner shows `pending_email`; old email still logs in; verification code arrives at the new email; code accepted → `users.email` swaps and the banner clears

### Settings → Security
- [ ] Change password requires current password; succeeds with valid input; current session stays live
- [ ] Sign out everywhere prompts for current password; wrong → inline error; right → all sessions on all devices end

### Settings → Delete account (M7)
- [ ] Delete card hidden in guest mode
- [ ] Initial click expands the confirm pane; warning copy + password field render
- [ ] "Delete forever" stays disabled until the password field is non-empty
- [ ] Wrong password → inline error, account untouched
- [ ] Correct password → routes to `/login?deleted=1`; "Account deleted." note shows
- [ ] Note clears once the user types in the email field
- [ ] Re-login attempt with deleted credentials → standard "Invalid credentials"
- [ ] DB inspection: `account_deletions` row exists with the email + timestamp; `users` row gone; sessions / settings / progress for the deleted user gone
- [ ] Other user's rows untouched
- [ ] Rate limit smoke: 4 wrong delete passwords in a row → 429 on the 4th
- [ ] **Known bug to verify:** delete + re-register same email → fresh streak starts at 0

---

## 2. Cases (browser + detail)

### Browser
- [ ] `/cases` shows all 57 cases as a flat grid sorted by case number
- [ ] Each tile shows the pattern diagram, zero-padded number, nickname (when set), and a state pip
- [ ] Search filters on number, nickname, algorithm, and tags (case-insensitive)
- [ ] Tier-1 chip filter (`All`/`Dot`/`L`/`Line`/`Cross`) narrows correctly
- [ ] Tag chip row narrows on any-of selected tags
- [ ] **State chip row** narrows on selected states (multi-select; empty = no filter)
- [ ] "Free study →" button carries current filters (tier1 / tags / state) through to `/free-study` via query params
- [ ] Empty filter result → "No cases match" empty state with a "Clear filters" CTA

### Detail / edit
- [ ] Tile tap → `/cases/:id` renders pattern, algorithm, tier-1 chip, tags, result-after-algorithm preview
- [ ] Edit mode: changing nickname → save persists across reload; tile shows the override
- [ ] Edit mode: clearing every field → `user_case_settings` row gone (`has_overrides=false`)
- [ ] Edit mode: changing `result_case_id` to a valid case + new rotation → preview updates; save persists
- [ ] Edit mode: setting `result_case_id` to an out-of-range integer → form-level validation error
- [ ] Tag editing: comma-separated input adds free-form tags; saving an empty input clears tags (override → empty array)
- [ ] Tag cap: attempting to add a 101st distinct tag → "Too many tags" form error
- [ ] **Bottom Save button** present alongside the top-right Save when in edit mode
- [ ] Cross-user isolation: user A's tag override on case 02 doesn't appear in user B's view
- [ ] Default global nicknames are NULL (M5/M9) — fresh accounts see "Unnamed" until they pick their own
- [ ] Case 34 (`Upstairs`): result diagram shows Case 35 rotated 90° CW (not CCW) — verifies migration 0008
- [ ] PatternDiagram renders correctly across cases that exercise all five letters and at least one rotated case

---

## 3. Study loop

### Practice tab
- [ ] Fresh account: "Nothing due" state with "Free study →" CTA (not "Browse cases")
- [ ] Streak shows 0
- [ ] When cards are due, the queue card shows a count and "Begin session →"

### In-session
- [ ] Tap "Begin session" → `/study` opens with the pattern at large size (240px) and "Reveal answer"
- [ ] Tap "Reveal answer" → pattern shrinks to ~120px and the reveal blocks show
- [ ] **Reveal order**: Should become → How did it go? → Algorithm
- [ ] Algorithm block has an "Edit →" link
- [ ] Tap Edit → routes to case detail in edit mode (`?from=study`)
- [ ] Save in edit mode → returns to `/study` on the same card with reveal hidden, showing the new algorithm/result on next reveal
- [ ] Cancel in edit mode → also returns to `/study` on the same card
- [ ] Grade buttons: Fail (red), Hard (amber), Good (blue), Easy (green) — colors swapped from earlier builds
- [ ] Tap Good → progress dot fills, queue advances or session-complete view appears
- [ ] End-session × button mid-queue exits cleanly without recording the un-graded current card
- [ ] Direct nav to `/study` while signed out → redirects to `/login?next=/study`
- [ ] Direct nav to `/study` with no active queue → bounces to `/practice`

### Session order
- [ ] Same set of due cards on two separate sessions on the same day → cards appear in different orders (shuffle on every start)
- [ ] Free-study session → cards in shuffled order, not by case number

### Session complete
- [ ] Tally renders with grade counts
- [ ] Primary "Repeat session" button re-runs the same set in a fresh shuffled order
- [ ] Secondary "Back to practice" button ends the session and returns to `/practice`

### SM-2 behavior
- [ ] First Good on a `not_started` card → state pip flips to `learning`, due tomorrow
- [ ] Submit Fail on a card with reps > 1 → reps reset to 0, interval = 1, ease drops by 0.20 (verify via psql)
- [ ] After 5+ Goods on a single card crossing 21-day interval → state pip flips to `mastered`

### Streak
- [ ] First grade on a fresh account → streak ticks 0 → 1
- [ ] Practice on consecutive days → streak increments by 1 per day
- [ ] Skip a day, then practice → streak resets to 1

---

## 4. Free study

- [ ] Reachable from: PracticeView empty state, PracticeView "Free study" link, CasesView "Free study →" header button
- [ ] Filter axes: Primary shape, Tags (when any exist), State
- [ ] **Primary shape: only / any-of toggle** present, defaults to "only"
- [ ] **Tags: only / any-of toggle** present, defaults to "any-of"
- [ ] **State: only / any-of toggle** present, defaults to "any-of"
- [ ] Live "N cases match" count updates as filters change
- [ ] Filters carried in via `/free-study?tier1=…&tags=…&state=…` apply on mount
- [ ] "Begin session →" disabled when 0 cases match
- [ ] Mid-session, fail/grade behavior identical to Practice mode

---

## 5. Dashboard / Progress

### Practice (signed in, with reviews)
- [ ] Streak KPI card + Due count card render
- [ ] "Today's queue" card shows oldest-first session entry when due > 0
- [ ] Standing card shows segmented bar + 4 chips (mastered / learning / due / not-started) summing to 57
- [ ] **Each Standing chip is clickable** and routes to `/cases?state=<key>` with the matching state chip pre-selected

### Progress view
- [ ] State chip row at the top
- [ ] Tap "due" → list filters to due cases
- [ ] Bottom of progress: "Stats over time — coming soon" placeholder visible
- [ ] After a review session → counts update without a hard refresh

---

## 6. Guest mode (M6)

### Entry
- [ ] Landing page hero: primary CTA is "Continue as guest →" (top-right Sign-in link + bottom Sign-in link still visible)
- [ ] LoginView footer also has a "Continue as guest →" link
- [ ] Tap "Continue as guest" → routes to `/welcome` (onboarding stub, step 1)
- [ ] Complete onboarding → lands on `/practice`

### Banner
- [ ] **Top-of-page banner** above the page content, scrolls away with content (not a fixed overlay)
- [ ] Visible on Practice / Cases / Progress for guests; hidden in authed mode
- [ ] × dismisses for the session; reopens next session
- [ ] "Hide until 10 reviews" suppresses until `progress.size >= 10`

### Guest study
- [ ] Browse `/cases` → all 57 render
- [ ] Open one → algorithm + result diagram render
- [ ] Edit a case (nickname / algorithm / result mapping / tags) → reload → edit persists
- [ ] Add a free-form tag → tag chip appears in the cases-page filter row
- [ ] Try to add a 101st tag → form rejects with "Too many tags"
- [ ] Run a study session → SM-2 schedules locally → next card. Reload mid-session → bounces back to `/practice` cleanly
- [ ] Streak ticks on grade across simulated days (clear `last_practice_date` in localStorage between)
- [ ] Free study + Progress view both reflect the localStorage state

### Upgrade
- [ ] Banner "Save your progress →" → `/upgrade`
- [ ] Settings → Save your progress card visible only in guest mode
- [ ] Submit upgrade form → server creates account + imports data + clears `oll-guest-state` → verify-email screen
- [ ] Verify email → onboarding does NOT re-fire; Practice tab shows the imported progress
- [ ] On a different browser, sign in → progress matches

### Merge-on-login
- [ ] Build a fresh guest blob with some progress, then sign in to a different existing account → merge prompt renders
- [ ] "Merge into this account" → progress folds in (max-rule); blob cleared
- [ ] "Discard" on a different test → blob cleared, no merge
- [ ] Server interval=10 / guest interval=2 → after merge server stays at 10
- [ ] Server interval=2 / guest interval=10 → after merge server takes 10

### Storage hygiene
- [ ] Sign out of upgraded account, click "Continue as guest" again → fresh blob (cleared on upgrade)
- [ ] `localStorage` size after dense use stays < 100 KB (DevTools)

---

## 7. Static pages, polish, route guards

### Landing
- [ ] `/` while signed out → landing page renders
- [ ] Top-right "Sign in" routes to `/login`
- [ ] "Create an account" CTA routes to `/register`
- [ ] Footer links: About / Terms / Privacy / Acknowledgements all reachable
- [ ] `/` while signed in → automatic redirect to `/practice`

### Static pages
- [ ] All four static pages (`/about`, `/terms`, `/privacy`, `/acknowledgements`) render with placeholder copy + back button that returns to the previous route
- [ ] Settings → About card links work

### Onboarding
- [ ] First successful verification of a new account routes to `/welcome` (step 1)
- [ ] "Next →" advances to step 2; "Got it →" lands on `/practice` and flips `has_seen_onboarding=true`
- [ ] Skip link also flips the flag and routes
- [ ] Sign out + sign back in → no re-route to `/welcome`
- [ ] Sign in on a different browser as the same user → no re-route to `/welcome` (backend flag, not localStorage)
- [ ] `UPDATE users SET has_seen_onboarding = false WHERE id = …` and re-verify → onboarding reappears

### Empty states
- [ ] Fresh account on PracticeView: empty state visible, Standing card hidden, "Free study →" CTA works
- [ ] Fresh account on ProgressView: empty state copy + CTA visible
- [ ] CasesView with a filter that matches nothing: empty state with "Clear filters" CTA

### 404
- [ ] `/does-not-exist` while signed in → renders `<NotFoundView>` with "Back to practice" CTA
- [ ] Same path while signed out → CTA reads "Back to login"

### Mobile
- [ ] iOS Safari (latest) — every page scrolls cleanly, no horizontal overflow, tab bar doesn't cover bottom content
- [ ] Android Chrome (latest) — same
- [ ] Desktop Chrome at 375px width — layout matches mobile expectations
- [ ] Practice / Progress views: bottom content not clipped by tab bar (90px bottom padding fix)

### Accessibility
- [ ] Tab key reaches every interactive element on dashboard, study session, settings
- [ ] Visible focus ring on every focusable element (chips, buttons, inputs, RouterLinks)
- [ ] Every form input has a `<label>` or `aria-label`
- [ ] Color contrast spot-check: paper-ink on paper-bg, paper-accent on paper-bg, paper-ink-muted on paper-card all meet AA

---

## 8. Backend smoke

Run from psql against the deployed Neon instance.

- [ ] `SELECT count(*) FROM cases WHERE solve_stage_id = (SELECT id FROM solve_stages WHERE name='OLL')` → 57
- [ ] `SELECT count(*) FROM puzzle_types` → 1; `SELECT count(*) FROM solve_stages` → 1
- [ ] `SELECT count(*) FROM cases WHERE nickname IS NOT NULL` → 0 (verifies migration 0009)
- [ ] `SELECT result_rotation FROM cases WHERE case_number = 34` → 1 (verifies migration 0008)
- [ ] `SELECT count(*) FROM user_case_progress` matches the number of distinct (user, case) pairs you've reviewed across test accounts
- [ ] `SELECT case_number, tags FROM cases ORDER BY case_number LIMIT 5` returns array values, not single strings
- [ ] After a delete: row in `account_deletions(email, deleted_at)` matches the deletion
- [ ] FK introspection (already covered by `tests/users_cascade_schema.rs` in CI) — every FK to `users.id` has `ON DELETE CASCADE`

---

## 9. Pre-launch content gates

These aren't really QA — they're the content tasks that block public launch. Listed here so the QA pass can flag them red until they're filled in.

- [ ] Terms of Service real content lives at `/terms` (not placeholder)
- [ ] Privacy Policy real content lives at `/privacy` (not placeholder)
- [ ] OLL case numbering universality verified (1–57 matches the cubing-community convention)
- [ ] Onboarding screen final copy + design landed
- [ ] Landing page final copy landed
- [ ] Splash screen final treatment confirmed
