# Milestones

A high-level split of the MVP into testable, shippable phases. Each milestone leaves the app in a deployable state — the goal is "always green" rather than a single big-bang launch. Per-milestone deep-dives (acceptance tests, ticket breakdowns, sequencing within the milestone) live in sibling files once a milestone is in flight.

---

## Ordering rationale

1. **Auth first.** Account flows are the riskiest non-app concern — email deliverability, JWT cookies, reCAPTCHA, argon2 hashing, the Resend integration, the Neon connection. Getting them working early on a real deployed instance means everything else is built on a known-good foundation. It also forces the "is the deploy pipeline working?" question to be answered up front.
2. **Data before behavior.** All 57 cases ship and render before any review/scheduling logic lands. Confidence the case data is right and the diagrams render is a prerequisite for trusting the SM-2 numbers later.
3. **Core loop before periphery.** Study mode + grading + scheduling lands before the dashboard, free study, tags, and stats. The latter are mostly views over data the core loop produces.
4. **Guest mode last.** Per `OLL_App_Design_Doc.md` §1 and `guest_mode_design_doc.md`, guest mode is the final MVP feature, layered on top of the authenticated path.

---

## Milestone 1 — Auth & Accounts

**Goal:** Everything account-related works end-to-end against a deployed instance. Nothing app-specific in the way.

**Done when:**
- Register → verify email with 6-digit code → land in (placeholder) dashboard
- Sign in, sign out
- Forgot password → reset code → log in with new password
- Change password while signed in
- Edit display name; change email with the `pending_email` flow + verify banner
- Sign out everywhere (with current-password confirmation)
- Resend verification code
- Route guard redirects unauthenticated users to `/login?next=…` and authenticated users away from auth-only routes
- Splash holds for ≥800ms while `/auth/me` is in flight
- All flows tested against the real deployed Render + Neon + Resend stack

**Backend:** Rust + Axum scaffold, sqlx migrations for `users` and `sessions`, all `/auth/*` endpoints, argon2id, JWT httpOnly cookie, reCAPTCHA v3 verification, Resend integration, tracing.

**Frontend:** Vue + Vite + Vue Router + Pinia scaffold, `tokens.css` ported from the prototype palette/fonts. Splash, Login, Register, VerifyEmail, ForgotPassword, ResetPassword, Settings (account + security sections). Route guard with `next` param. Tab bar exists structurally but the three tabs are placeholders.

**Pre-work / parallel:** Resend domain verification kicked off ASAP (1–2 business day lead). reCAPTCHA v3 site key + secret key registered. Neon DB and Render frontend/backend services provisioned.

**Out of scope:** Any case data, study, scheduling, progress, tags, free study, guest mode, real static page content.

---

## Milestone 2 — Case Data & Browser

**Goal:** All 57 OLL cases live in the database, render correctly, and can be browsed and per-user overridden.

**Done when:**
- All 57 cases seeded from `initial_design/src/data.jsx`
- `<PatternDiagram>` ported to Vue and renders cases with the correct rotation for back-of-card
- Case browser shows the full grid sorted by case number; case detail shows algorithm + result diagram
- User can edit nickname / algorithm / result mapping / Tier 2 tag (writes to `user_case_settings`); fields default to the global value via the override-merge logic on the backend

**Backend:** Migrations for `puzzle_types`, `solve_stages`, `cases`, `user_case_settings`. Seed script for the 3×3 puzzle type, OLL stage, and all 57 cases. `GET /cases`, `GET /cases/:id` with override merge, `PATCH /cases/:id/settings`.

**Frontend:** Cases tab + case detail. Read-only fields fall back to global defaults; edit form writes user overrides. PatternDiagram component with `rotation` prop.

**Out of scope:** SM-2 / scheduling / grading. Free-form tags. Free study filters.

---

## Milestone 3 — Core Study Loop (SM-2)

**Goal:** The meat and potatoes — a user can study due cards, grade them, and have the Anki-variant SM-2 schedule update correctly. Streak ticks.

**Done when:**
- Reviewing a card runs SM-2 (per `OLL_App_Design_Doc.md` §4) → `due_date` updates → streak updates per the day-rollover rule
- Practice tab pulls due cards and runs the full study session (pattern → reveal algorithm → 4-button grade → next)
- Failing a card resets `repetitions` and `interval_days` correctly; Hard/Easy modify ease per the constants table
- Cards transition through `not_started` → `learning` → `due` → `mastered` per the §1.3 thresholds
- SM-2 module has full unit-test coverage on the Rust side, including edge cases (rep 0, rep 1, ease floor, easy bonus, fail reset)

**Backend:** Migration for `user_case_progress`. SM-2 algorithm module + tests. `GET /study/due`, `POST /study/:case_id/review`. Streak update logic on review submission.

**Frontend:** Practice tab → study session screen with the 4-button rating UI (Fail / Hard / Good / Easy); reveal-then-grade flow; streak count and due count visible on the (still-stub) dashboard.

**Out of scope:** Free study (with filters), per-case progress view, tags, dashboard polish beyond streak + due count.

---

## Milestone 4 — Dashboard, Progress, Free Study, Tags

**Goal:** Every authenticated view in the spec is functional. The app works end-to-end for a logged-in user.

**Done when:**
- Dashboard shows streak, due-today, learning/mastered counts, quick-start CTA
- `/progress` shows the per-case breakdown with state filters (`not_started` / `learning` / `due` / `mastered`)
- Free study mode runs with all four filter axes (Tier 1, Tier 2, user tag, status)
- User-defined tags: create / delete / apply / remove, all from the case-detail screen
- "Stats over time" skeleton renders a "coming soon" panel on `/progress`

**Backend:** `GET /progress` (with the stats fields per item E of the auth decisions doc), `GET /progress/cases`, `GET /study/free`, full `/tags` and `/cases/:id/tags` CRUD.

**Frontend:** Dashboard, Progress view, Free Study view + filter UI, Tags UI on case detail, stats skeleton.

**Out of scope:** Static pages, guest mode, polish/empty states.

---

## Milestone 5 — Polish & Static Pages

**Goal:** Everything that isn't a feature but is required for launch quality.

**Done when:**
- `/about`, `/terms`, `/privacy`, `/acknowledgements` are live (placeholder content OK during this milestone; real Terms/Privacy content per `docs/TODO.md` lands before launch)
- Footer / settings links wire up correctly
- Empty states for fresh accounts: zero-reviews dashboard, no-due-cards study tab, etc.
- Banners: email-change pending verification, post-error toasts
- Onboarding flow stub (designer's deeper work tracked in `docs/TODO.md`)
- Mobile QA on real devices (mobile-shaped layout on desktop confirmed acceptable)
- Accessibility pass — keyboard nav, contrast, labeled inputs

**Backend:** No new endpoints.

**Frontend:** Static page views, empty-state components, banner components, error boundaries.

**Out of scope:** Guest mode.

---

## Milestone 6 — Guest Mode

**Goal:** Final MVP feature per `docs/guest_mode_design_doc.md`.

**Done when:**
- Guest persistence in `localStorage` with the versioned schema (v1)
- All four core flows (study, free study, cases, progress) work in guest mode via a Pinia adapter that dispatches to `localStorage` when `authStore.isGuest`
- `GuestUpgradeScreen` migrates guest data to a new account on registration (`guest_state` field on `POST /auth/register`)
- Merge-on-sign-in flow handles the "I already have an account on another device" path with a max-of-(server, guest) merge via `POST /auth/merge-guest-state`
- The 7 open questions in `guest_mode_design_doc.md` §8 are resolved before implementation begins

**Out of scope:** —

---

## Parallelization opportunities

The milestones above are listed in the natural completion order, but some work can run in parallel without breaking the "always green" rule:

- **M2 case-data seeding** can begin during M1 — the dataset and the diagram component don't depend on auth.
- **M5 static page placeholders + onboarding stubs** can be added during M4. The polish pass itself stays at the end.
- **Real Terms / Privacy content** is on the user's plate (`TODO.md`) and can land at any time once written.

Things that should *not* be parallelized:

- M3 before M2 (no case data → no study).
- M6 before M5 — guest mode replays every flow we ship through the localStorage adapter. Easier to wire that adapter up after the canonical paths are stable.

---

## Deployment posture

Render + Neon + Resend are live from M1 onward. Each milestone's "done when" is verified against the deployed environment, not a local-only build. CI deploys on green main per `OLL_App_Design_Doc.md` §10. No big-bang launch — by the end of M6 the deployed app *is* the MVP.

---

## After M6

Out of scope for this document. The post-MVP list in `OLL_App_Design_Doc.md` §1 (PLL/F2L, stats over time, admin panel, public case browser, dark mode) is the source of truth for what comes next.
