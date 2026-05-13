# Changelog

A short, dated log of what shipped — the long-term tracker that survives
after milestone docs move into `milestones/retro/`. New entries land at
the top. Each entry is a sentence or two; depth lives in the milestone
docs and in `git log`.

Dates are calendar dates of the merge that made the slice live on
[cube.nebulouscode.com](https://cube.nebulouscode.com), drawn from the
git history.

---

## 2026-05-07 — M7: Account deletion (MVP complete)

`DELETE /auth/me` with password gate, two-step confirm in Settings, and
an `account_deletions` audit table written transactionally with the
delete. Backend test coverage hit ~98% on the testable surface; CI gate
sits at 95%. Closes the last MVP milestone.

## 2026-05-05 — M6: Guest mode

Localstorage-backed guest blob, frontend Pinia adapter that branches the
study / cases / progress flows on `auth.isGuest`, and a backend
`POST /auth/merge-guest-state` for upgrade-on-sign-in. The full study
loop runs without an account; upgrade folds guest data into a fresh
user.

## 2026-05-05 — M5: Polish + static pages

Landing page, About / Terms / Privacy / Acknowledgements (placeholder
copy), onboarding stub, empty states, 404, focus-ring + label/aria pass.
Tab-bar bottom-padding fix; mobile QA pass. Onboarding gate driven by
`users.has_seen_onboarding`.

## 2026-05-03 — M4: Dashboard, progress, free study, tags

Standing card, progress view with state filter, free-study filter screen
(primary shape / tags / state with only/any-of toggles). Tag rework
collapsed the planned `tags` + `case_tags` junction tables into a
`tags TEXT[]` column on `cases` and `user_case_settings`.

## 2026-04-29 — M3: Core study loop (SM-2)

Anki-variant SM-2 in `srs/`, study session UI with reveal + 4-button
grade, streak update on the day-rollover rule. Cards transition
`not_started → learning → due → mastered` per the §1.3 thresholds.

## 2026-04-29 — M2: Case data + browser

All 57 OLL cases seeded from the prototype's `data.jsx`. PatternDiagram
ported to Vue. Case browser with filters; case detail with per-user
override editing (algorithm / nickname / result mapping / Tier 2 tag).

## 2026-04-28 — M1: Auth + accounts

Email/password registration with reCAPTCHA, 6-digit verification codes
via Resend, login/logout/forgot/reset, change-password, sign-out-all
with current-password gate, email-change `pending_email` flow, JWT
httpOnly cookies signed HS256. Deployed to Render (frontend + backend)
backed by Neon Postgres, reachable at cube.nebulouscode.com.

---

## Pre-M1 (2026-04-27 → 2026-04-28)

Initial design pass. The original spec, SM-2-vs-Anki research, and the
auth-decisions Q&A live in
[`milestones/00_initial_design_doc.md`](milestones/00_initial_design_doc.md)
and [`concepts/sm2_vs_anki_summary.md`](concepts/sm2_vs_anki_summary.md).
