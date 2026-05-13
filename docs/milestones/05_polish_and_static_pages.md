# Milestone 5 — Polish & Static Pages

Detailed design + story list for M5. Scope set in `docs/milestones/README.md`. Authoritative spec is `docs/Cube_Practice_Design_Doc.md` §1 (MVP feature list, mobile-friendly), §10 (deployment posture), and `docs/TODO.md` (real legal copy is on the user's plate, separate from this milestone).

---

## 1. Goal recap

By the end of M5:

- A public landing page lives at `/` — what Quiet Cube is, who it's for, and a "Sign in" CTA front-and-center for existing users (also a top-right link). The authed dashboard moves from `/` to `/practice`.
- Every static page in the spec is reachable, styled, and linked: `/about`, `/terms`, `/privacy`, `/acknowledgements`. Placeholder copy is fine — real Terms / Privacy content lands separately per `docs/TODO.md`.
- Registration screen carries the "By creating an account you agree to our Terms…" footer with working links to `/terms` and `/privacy`.
- Every authenticated view has a polished fresh-account empty state, not a blank/zero rendering. Specifically: dashboard with zero reviews, study tab with zero due cards, progress with all-not-started, cases browser with empty filter results.
- Onboarding stub — a two-step welcome screen shown once after first email verification. Real designer-driven content lands later per `docs/TODO.md`; this milestone ships a minimal skeleton so the wiring is in place. Seen-state lives on the backend (`users.has_seen_onboarding`) so it survives device changes and `localStorage` clears.
- Catch-all 404 route renders a real "Not found" view rather than a blank page or silent redirect.
- Mobile QA pass on real iOS Safari + Android Chrome. Mobile-shaped layout on desktop confirmed acceptable per the design doc.
- Accessibility pass — keyboard nav reaches every actionable element, focus rings are visible, form inputs are labelled, color contrast meets WCAG AA on the paper palette.

Out of scope (deferred):
- Real Terms / Privacy / Acknowledgements **content**. Tracked on `docs/TODO.md`. Lands before public launch independently of this milestone.
- Designer-driven onboarding visuals. Stub only — designer replaces post-M5 per `docs/TODO.md`.
- Guest mode — M6.
- Auto-generated acknowledgements from `package.json` / `Cargo.toml` license metadata. Post-MVP per `docs/TODO.md`.

---

## 2. Architecture for M5

Almost all frontend / UX. One small backend addition: a boolean column on `users` and an endpoint to flip it.

### Backend additions
- Migration `0006_users_onboarding_seen.sql` adding `has_seen_onboarding BOOLEAN NOT NULL DEFAULT FALSE` to `users`.
- `User` struct (and `/auth/me` response) carry the new field.
- `POST /auth/onboarding-complete` — auth-gated, idempotent, sets the flag to TRUE for the calling user. No body, no response body beyond `{ ok: true }`.

### Frontend additions
- **`<LandingView>`** at `/` (public). Marketing-style page with the wordmark + tagline, two CTAs (Sign in / Create account), a small feature list, and a footer linking to the static pages. Authed users hitting `/` are redirected to `/practice`.
- **Route reshuffle.** The authed tab shell moves from `/` to `/practice`. `/cases` and `/progress` stay where they are (still children of `AppShell`, just under the new parent path). The tab bar's Practice tab updates its `to` from `/` to `/practice`. Login-success and post-onboarding redirects update from `/` to `/practice`. The router guard's default `next` for unauthenticated visits to a `requiresAuth` route stays as today (preserves the original path in `?next=`).
- **`<EmptyState>` component.** A reusable slot wrapper that renders an italic serif headline, an optional descriptive paragraph, and an optional CTA button. Used by every view's empty branch so the visual treatment is consistent.
- **`<NotFoundView>`** at the catch-all `:pathMatch(.*)*` route. Same `<EmptyState>` shell — "Nothing here" + "Back to practice" button.
- **`<OnboardingView>`** at `/welcome`. Two-step skeleton (intro card with "Practice with intention" + "Weakest cases come first" copy, "Next →" / "Got it →" buttons). Triggered exactly once via the `VerifyEmailView` success handler when `auth.user.has_seen_onboarding === false`. Final step calls `POST /auth/onboarding-complete` then routes to `/practice`.
- **Registration footer.** Two `<RouterLink>`s in the existing `RegisterView.vue` below the CTA: legal copy + Terms + Privacy.

### Reuse from M1–M4
- `StaticPageView.vue` — already shipped in M1. All four legal/about pages already extend it. M5 leaves the wrapper alone and just confirms the routes are reachable from registration + settings.
- `PendingEmailBanner.vue` — already in `AppShell`. M5 doesn't touch it.
- The existing inline `.error` form pattern stays for form-level validation across the board. No global notification surface.
- Splash ≥800ms minimum hold — already in `stores/auth.ts:74`. No change needed.

### What's already done (validated during M5 survey)
- All four static page views exist with placeholder copy and `StaticPageView` styling.
- `SettingsView` → About card links to all four.
- `PendingEmailBanner` covers the email-change-pending path.
- Splash min-hold prevents the cold-start flicker.
- `PracticeView` already has a "nothing waiting" empty state for the queue card.

So M5 is genuinely about closing the gaps: landing page + route reshuffle, registration footer, missing empty states, onboarding stub (with the small backend flag), 404, and the QA passes.

---

## 3. Schema — M5 changes

### Migration `0006_users_onboarding_seen.sql`

```sql
ALTER TABLE users
    ADD COLUMN has_seen_onboarding BOOLEAN NOT NULL DEFAULT FALSE;
```

Existing users get `FALSE` on the backfill — meaning they'll see onboarding the next time they verify and land. Acceptable: there are no production users yet. If we ship this after launch we'd want to backfill `TRUE` for accounts created before a cutoff timestamp; not relevant pre-launch.

---

## 4. API surface — M5 additions and changes

Prefix `/api/v1`. Auth-gated unless noted.

### New

| Method | Endpoint | Body | Returns |
|--------|----------|------|---------|
| POST | `/auth/onboarding-complete` | — | `{ ok: true }` |

Idempotent: setting `has_seen_onboarding = TRUE` on an already-true row is a no-op. No 4xx beyond the standard auth gate.

### Changed shape

- `/auth/me` (and every other endpoint that returns the `User` shape — register, login, profile update) now includes `has_seen_onboarding: bool` on the JSON. Pre-existing clients ignore unknown fields, so this is safe to ship in either order.

---

## 5. Frontend — M5

### Routes

| Path | Auth | Notes |
|------|:---:|------|
| `/` | none | New `LandingView`. Authed users redirected to `/practice` via the guard. |
| `/practice` | required | New home for `PracticeView` (was `/`). Default landing for authed users. |
| `/cases` | required | Unchanged. |
| `/progress` | required | Unchanged. |
| `/welcome` | required | Onboarding skeleton — first-run only |
| `/:pathMatch(.*)*` | none | Catch-all 404 |

Existing static-page routes (`/about`, `/terms`, `/privacy`, `/acknowledgements`) and auth routes (`/login`, `/register`, etc.) stay where they are.

### Components / views

- **`<LandingView>`** at `/` — public marketing page. Layout, top to bottom:
  1. **Top bar.** Wordmark on the left ("Quiet Cube"), single "Sign in" link on the right (routes to `/login`).
  2. **Hero.** Centered. Logo mark above the title. Title + tagline + two CTAs side by side: primary "Sign in →" (routes to `/login`), secondary "Create an account" (routes to `/register`). This is the front-and-center login surface for existing users.
  3. **Feature list.** Four short bullets describing what's in the app today.
  4. **How it works.** Three numbered steps, kept brief.
  5. **Closing CTA.** A second "Sign in →" so the existing-user path is reachable without scrolling back to the top on mobile.
  6. **Footer.** Small links: About · Terms · Privacy · Acknowledgements.

  Authed users hitting `/` are redirected to `/practice` by the router guard (see route reshuffle in §2). No tab bar, no settings gear — this is a guest-facing surface.

  **Placeholder copy** (replace before launch — tracked in `docs/TODO.md`):

  > **Quiet Cube**
  > *a quiet place to drill*
  >
  > Spaced repetition for Rubik's cube algorithms. Build muscle memory for the cases you don't yet know, and keep the ones you do sharp.
  >
  > [Sign in →]   [Create an account]
  >
  > **What you get**
  > • All 57 OLL cases ready out of the box
  > • Anki-style SM-2 schedules each case for you
  > • Free study any case, any time
  > • Track your streak and what's due today
  >
  > **How it works**
  > 1. Pick a case to drill, or let the schedule pick for you.
  > 2. See the pattern, recall the algorithm, then grade yourself.
  > 3. The schedule decides when each case comes back around.
  >
  > [Sign in →]
  >
  > About · Terms · Privacy · Acknowledgements

- **`<EmptyState>`** — Slot-based: `<template #icon>`, `<template #title>`, `<template #body>`, `<template #cta>`. Uses paper-card background, italic serif title, sans body copy at 13px. Centered vertically inside the page area.

- **`<OnboardingView>`** at `/welcome` — Two consecutive cards swapped in via `step` ref. Step 1: "Practice OLL with intention" + brief paragraph + "Next →". Step 2: "Weakest cases come first" + brief paragraph + "Got it →" which calls `auth.completeOnboarding()` (which `POST`s to `/auth/onboarding-complete` and updates `auth.user.has_seen_onboarding` locally) then routes to `/`. Skip link "Skip onboarding" in the corner does the same.

  Trigger logic lives in the `VerifyEmailView` success handler. When verification succeeds and `auth.user.has_seen_onboarding === false`, route to `/welcome` instead of `/`. The router guard does **not** check the flag — onboarding only fires once at first verification. Existing users on a new device skip onboarding (the backend already remembers they've seen it).

- **`<NotFoundView>`** at the catch-all — Uses `<EmptyState>`. Title "Nothing here." Body "The page you're looking for doesn't exist." CTA "Back to practice" → `/` (or `/login` if unauthed — checked via `authStore.status`).

### Empty-state coverage

A pass over each view to ensure the zero-data path renders the `<EmptyState>` component, not raw zeros or a blank list:

| View | Empty case | Treatment |
|------|------------|-----------|
| `PracticeView.vue` | dueCount=0 | Already has `nothing waiting` copy + Browse cases CTA. Wrap in `<EmptyState>` for consistency. Standing card hides while standingTotal=0 (already conditional). |
| `StudySessionView.vue` | queue empty (shouldn't happen mid-session) | Defensive `<EmptyState>` "Nothing to study." + back button. |
| `ProgressView.vue` | all `not_started` (fresh account) | "Pick a case to start" copy + CTA to Cases tab. The list still renders below — empty state appears above list when total reviewed = 0. |
| `CasesView.vue` | filtered to zero | Existing `state` div — keep as-is, just wrap in `<EmptyState>` with a "Clear filters" CTA. |
| `FreeStudyView.vue` | matchCount=0 | Already disabled CTA. Add a small note under the count: "No cases match — loosen filters." |

### Registration footer

In `RegisterView.vue`, below the existing CTA:

```html
<p class="legal">
  By creating an account you agree to our
  <RouterLink to="/terms">Terms</RouterLink> and
  <RouterLink to="/privacy">Privacy Policy</RouterLink>.
</p>
```

Style: 11px sans, `--paper-ink-faint`, links underlined with `--paper-rule-faint`. Shows for all users, every render.

### Notification surface — none

We deliberately don't ship a global toast/popup system in M5. The existing inline `.error` / `.note` form pattern handles every site that needs feedback today. The one currently-silent path (`progressStore.reload()` failure post-review) is acceptable as-is — the next successful action refreshes it. If a real need shows up later, we'll revisit.

---

## 6. Security notes specific to M5

- Static page links are `<RouterLink>` to internal routes — no external redirects, no XSS surface.
- `POST /auth/onboarding-complete` is auth-gated and only ever writes the calling user's row. No way to flip another user's flag.
- Catch-all 404 route is unauthenticated — it must not leak any user data. Confirmed: `<NotFoundView>` reads only `authStore.status` to decide which CTA to show; renders no user content.

---

## 7. Testing strategy

**Backend (cargo test):**
- Migration test: existing users get `has_seen_onboarding = FALSE`.
- `POST /auth/onboarding-complete` integration tests: flips the flag for the calling user; idempotent on a second call; doesn't touch other users' rows; rejected when unauthed.
- `/auth/me` response carries the new field.

**Frontend unit (Vitest):**
- `<EmptyState>` snapshot with each slot combination.
- `auth.completeOnboarding()` action: posts to the endpoint, updates `auth.user.has_seen_onboarding` locally on success.
- Onboarding gate in `VerifyEmailView`: with/without the backend flag, redirect target.

**Manual QA:** §10 below.

---

## 8. Configuration / environment

No new env vars.

---

## 9. Migration / data risk

The `users.has_seen_onboarding` migration adds a `NOT NULL DEFAULT FALSE` column. The default makes the migration safe on a non-empty `users` table — every existing row gets `FALSE` and would re-see onboarding on next verification landing. Pre-launch this is a non-issue; if shipped post-launch we'd want a backfill condition based on `created_at`.

---

## 10. "Done when" checklist (run on the deployed instance)

A literal QA script. M5 is closed when every line passes against `https://cube.nebulouscode.com`.

- [ ] Visit `/` while signed out → landing page renders with hero CTAs and a top-right Sign in link.
- [ ] Top-right "Sign in" routes to `/login`. The big "Sign in →" hero CTA does the same.
- [ ] "Create an account" CTA on the landing page routes to `/register`.
- [ ] Footer links on the landing page route to About / Terms / Privacy / Acknowledgements.
- [ ] Visit `/` while signed in → automatic redirect to `/practice`.
- [ ] Direct hit on `/practice` while signed in → renders the dashboard. While signed out → redirects to `/login?next=/practice`.
- [ ] Tab bar's Practice tab links to `/practice`, highlights as active when on `/practice`.
- [ ] Login success routes to `/practice` when there's no `?next=` query.
- [ ] Registration screen shows the legal footer with working Terms + Privacy links.
- [ ] Tap Terms during registration: opens `/terms`, back button returns to register without losing form input.
- [ ] All four static pages render with placeholder copy and a back button that returns to the previous route.
- [ ] Settings → About card links work.
- [ ] First successful verification of a new account routes to `/welcome` (onboarding step 1).
- [ ] "Next →" advances to step 2.
- [ ] "Got it →" lands on `/` and `auth.user.has_seen_onboarding` flips to `true` (verify via DevTools / next `/auth/me`).
- [ ] Skip link on `/welcome` also flips the flag and routes to `/`.
- [ ] Sign out + sign back in on the same browser: no re-route to `/welcome`.
- [ ] Sign in on a different browser as the same user: no re-route to `/welcome`. Confirms the backend flag rather than localStorage.
- [ ] Reset the flag in the DB (`UPDATE users SET has_seen_onboarding = false WHERE id = …`) and re-verify: onboarding shows again. Confirms the gate is wired to the backend value.
- [ ] Fresh account (no reviews yet) on `PracticeView`: empty state visible, Standing card hidden, Browse cases CTA works.
- [ ] Fresh account on `ProgressView`: empty state copy + "Pick a case" CTA visible above the list.
- [ ] `CasesView` with a filter that matches nothing: empty state with "Clear filters" CTA.
- [ ] Catch-all 404: visit `/does-not-exist` while signed in → renders `<NotFoundView>` with "Back to practice" CTA. Same path while signed out → CTA reads "Back to login".
- [ ] Mobile QA — iOS Safari (latest): every page scrolls cleanly, no horizontal overflow, tab bar doesn't cover bottom content.
- [ ] Mobile QA — Android Chrome (latest): same.
- [ ] Mobile QA — desktop Chrome at 375px width: layout matches mobile expectations.
- [ ] Accessibility — `Tab` key reaches every interactive element on the dashboard, study session, settings.
- [ ] Accessibility — visible focus ring on every focusable element (chips, buttons, inputs, RouterLinks).
- [ ] Accessibility — every form input has an associated `<label>` (or `aria-label` on the chip rows where labels would be visually redundant).
- [ ] Accessibility — color contrast spot-check via DevTools: paper-ink on paper-bg, paper-accent on paper-bg, paper-ink-muted on paper-card all clear AA.

---

## 11. Story list

Pairs backend + frontend per the user's principle.

### Onboarding flag (backend + frontend)
- [ ] **B1.** Migration `0006_users_onboarding_seen.sql`. `User` struct + `/auth/me` payload gain `has_seen_onboarding: bool`. Test: existing rows default to FALSE.
- [ ] **B2.** `POST /auth/onboarding-complete` endpoint. Idempotent. Tests cover flip-once, idempotent-second-call, cross-user isolation, unauthed rejection.
- [ ] **D1.** `auth.completeOnboarding()` Pinia action — posts to the endpoint, updates `auth.user.has_seen_onboarding` on success. `User` interface gains the field.

### Empty-state primitive
- [ ] **D2.** `<EmptyState>` component with named slots (icon / title / body / cta). One Vitest snapshot per slot combination.

### Landing page + route reshuffle
- [ ] **D3a.** `<LandingView>` at `/` with the structure + placeholder copy in §5. Public route, no auth required.
- [ ] **D3b.** Move authed shell from `/` to `/practice`. Update `router/index.ts` (move the `path: ''` child to `path: 'practice'` and reparent under a top-level `/practice`-prefixed AppShell), `TabBar.vue` (Practice tab `to`), `LoginView.vue` (post-login default redirect from `/` to `/practice`), `OnboardingView` final routing, and any other internal `router.push('/')` calls that meant the dashboard. Add a `beforeEach` rule that redirects authed users away from `/` to `/practice`.

### Static pages + registration
- [ ] **D4.** Registration footer — Terms + Privacy links below the CTA in `RegisterView.vue`. No new components; uses existing `<RouterLink>` styling.

### 404
- [ ] **D5.** `<NotFoundView>` + catch-all route registered in `router/index.ts`. Renders `<EmptyState>` with auth-aware CTA.

### Empty states audit
- [ ] **D6.** Wrap empty branches in `PracticeView`, `ProgressView`, `CasesView`, `FreeStudyView`, and `StudySessionView` with `<EmptyState>`. Confirm copy + CTA per §5 table.

### Onboarding view + trigger
- [ ] **D7.** `<OnboardingView>` at `/welcome` — two-step skeleton + skip link. Both completion and skip call `auth.completeOnboarding()` then route to `/practice`.
- [ ] **D8.** Wire the trigger in `VerifyEmailView`'s success handler: when `auth.user.has_seen_onboarding === false`, route to `/welcome`; otherwise route to the post-verify default. No router-guard logic.

### Polish + a11y
- [ ] **D9.** Focus-ring pass — global `:focus-visible` styles in `tokens.css` so every chip/button/input gets a visible ring on keyboard focus. Verify across Landing, Practice, Cases, Progress, Free Study, Settings, study session.
- [ ] **D10.** Label/aria pass — audit every form input and icon-only button. Fix gaps (e.g., the gear icon in `AppShell` already has `aria-label="Settings"` — confirm similar coverage on the Cases search box, study-session reveal/grade buttons, and chip rows).

### QA
- [ ] **E1.** Walk §10 on the deployed instance, including mobile devices. Capture any regressions in `docs/TODO.md` if the fix slips out of M5.

---

## 12. Notes / open items

Resolved during planning — kept here as a record of the decisions:

1. **Onboarding copy.** Placeholder copy (the two-step "Practice with intention" / "Weakest cases come first" paragraphs in §5) ships as-is. Where to edit it: `frontend/src/views/OnboardingView.vue`, hard-coded in the template. Logged on `docs/TODO.md` for the user to swap when the designer ships final copy.

2. **Onboarding trigger surface.** Success-handler in `VerifyEmailView` only — not a router-guard. Onboarding fires exactly once, at first verification.

3. **Onboarding flag — frontend localStorage vs backend column.** Backend column (`users.has_seen_onboarding`). Survives device changes and `localStorage` clears.

4. **Notification surface.** No global toast/popup system. The existing inline `.error` / `.note` form patterns cover everything that needs feedback. The single currently-silent site (`progressStore.reload` failure post-review) stays silent — the next successful action refreshes the data. Revisit if a real need surfaces later.

5. **404 catch-all.** Render `<NotFoundView>` rather than silently redirecting. Keeps shared/typo'd URLs meaningful and gives a clear CTA back to safety.

6. **Empty state copy on `ProgressView`** — card-above-list with "Nothing reviewed yet. Pick a case to start your first session." CTA. The all-57-not-started list still renders underneath.

7. **Accessibility scope for M5.** Keyboard nav + visible focus rings + form labels + spot-check contrast. Screen-reader walkthrough and full WCAG audit are post-MVP — added to `docs/Cube_Practice_Design_Doc.md` §1 Post-MVP.

8. **Landing page / homepage at `/`.** Originally tagged as a post-MVP item; pulled into M5 since this is the static-page milestone. Authed dashboard relocates to `/practice`. Additional marketing pages (features, pricing, FAQ, etc.) remain post-MVP — added to `docs/Cube_Practice_Design_Doc.md` §1 Post-MVP.

---

## 13. Looking back

A retrospective pass added after the milestone shipped. Fill in honestly —
the value of these notes is the friction they capture, not a victory lap.

- **What I'd do differently if I started this milestone today:**

- **Surprises during execution:**

- **Decisions that turned out to matter more than expected:**

- **Decisions that turned out not to matter:**

- **What this milestone taught me that I'd carry into future work:**
