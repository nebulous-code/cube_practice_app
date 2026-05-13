# Milestone 3 — Core Study Loop (SM-2)

Detailed design + story list for M3. Scope set in `docs/milestones/README.md`. Authoritative spec is `docs/Cube_Practice_Design_Doc.md` §3 (`user_case_progress`), §4 (Anki-variant SM-2), and `docs/outstanding_decision.md` §1.3 (state thresholds).

---

## 1. Goal recap

By the end of M3:

- A user can open a case from the Cases tab, hit "Start studying," and immediately enter a one-card study session that creates the user's first `user_case_progress` row on grade submission.
- The Practice tab shows due cards, a count, and the current streak. Tapping "Start session" walks the user through every due card: pattern → reveal algorithm + result preview → 4-button grade (Fail / Hard / Good / Easy) → next.
- Submitting a grade runs the Anki-variant SM-2 update server-side and writes back the new `ease_factor` / `interval_days` / `repetitions` / `due_date`. Streak ticks per the day-rollover rule.
- Each merged `Case` carries a `state` field (`not_started` / `learning` / `due` / `mastered`) so the Cases browser can render a small state pip on every tile.

Out of scope (deferred):
- `/progress` route + dashboard breakdown — M4.
- Free study with filters — M4.
- Tags + tag-array rework — M4 (per the M2 follow-through).
- "Stats over time" chart skeleton — M4.
- Onboarding flow polish, empty-state design — M5.

---

## 2. Architecture for M3

### Backend
- One migration adding `user_case_progress` (per `Cube_Practice_Design_Doc.md` §3) plus a unique-`(user_id, case_id)` constraint and the shared `set_updated_at()` trigger.
- New module `backend/src/srs/mod.rs`. Pure SM-2 update function: takes the current row + the grade + today's date, returns the next row's `ease_factor` / `interval_days` / `repetitions` / `due_date`. Zero I/O — easy to unit-test exhaustively. The route layer wraps it with the DB read/write and the streak update.
- Extend `cases::merge` to LEFT JOIN `user_case_progress` so the merged `Case` carries `state`. Cheap addition: one more LEFT JOIN, one more derived column.
- New module `backend/src/study/mod.rs` housing `due_for_user()`, `apply_review()` (the DB wrapper around `srs::next_state()`), and the streak helper.
- New routes module `backend/src/routes/study.rs`: `GET /study/due`, `POST /study/:case_id/review`. Mounted under `/api/v1` alongside auth/cases.

### Frontend
- Replace `PracticeStubView.vue` with a real `PracticeView.vue` — header (streak + due count), "Start session" button if due > 0, gentle empty state if 0.
- New full-bleed route `/study` rendering `StudySessionView.vue` — pattern, reveal action, grade buttons, advance, session-complete summary.
- Extend `CaseDetailView.vue` with a state pip + "Start studying" button (visible when `state === 'not_started'`). Tap → navigates to `/study?case=<id>` for a single-card session.
- Extend `CaseTile` (in `CasesView.vue`) with a small state pip in the corner.
- New Pinia `studyStore` — manages the in-flight session queue, current index, and per-card grade history. Survives in-memory only (no localStorage); navigating away ends the session.

### Reuse from M1/M2
- Auth store, route guard.
- `casesStore` — for the case lookup during a session (algorithm, pattern, result preview).
- `PatternDiagram.vue`, `pattern.ts`.

---

## 3. Schema — M3 additions

### `user_case_progress`
```
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE
case_id         UUID NOT NULL REFERENCES cases(id)
ease_factor     DOUBLE PRECISION NOT NULL DEFAULT 2.5
interval_days   INT NOT NULL DEFAULT 1
repetitions     INT NOT NULL DEFAULT 0
due_date        DATE NOT NULL DEFAULT CURRENT_DATE
last_grade      INT
last_reviewed   TIMESTAMPTZ
created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()

UNIQUE (user_id, case_id)
CHECK (interval_days >= 1)
CHECK (repetitions >= 0)
CHECK (last_grade IS NULL OR last_grade BETWEEN 0 AND 3)
```

`updated_at` reuses the `set_updated_at()` trigger function from migration 0001.

Index: `CREATE INDEX user_case_progress_user_due_idx ON user_case_progress (user_id, due_date)` — supports the "due today" query.

The streak fields already exist on `users` (`streak_count INT`, `last_practice_date DATE`) from migration 0001.

### Per-user-per-case state derivation (no schema, query-time only)

| State | Rule |
|-------|------|
| `not_started` | No `user_case_progress` row |
| `due` | Row exists AND `due_date <= today` |
| `learning` | Row exists AND `due_date > today` AND `interval_days < 21` |
| `mastered` | Row exists AND `due_date > today` AND `interval_days >= 21` |

Implemented as a `CASE` expression in the merge SQL. `today` is the server's UTC date — see §12 for the timezone open item.

---

## 4. SM-2 update rule (Anki variant)

Authoritative source is `docs/Cube_Practice_Design_Doc.md` §4. Reproduced here for review-while-implementing convenience.

### Inputs
- Current `ease_factor: f64`, `interval_days: i32`, `repetitions: i32`.
- `grade ∈ {0, 1, 2, 3}` — Fail / Hard / Good / Easy.
- `today: NaiveDate`.

### Constants
| Constant | Value |
|----------|------:|
| Initial `ease_factor` | 2.5 |
| `EASE_FLOOR` | 1.3 |
| `HARD_INTERVAL_MULT` | 1.2 |
| `EASY_BONUS` | 1.3 |
| Fail ease delta | −0.20 |

### Update rule (pseudocode)
```
if grade == 0 (Fail):
    repetitions = 0
    interval_days = 1
    ease_factor -= 0.20
else:
    if repetitions == 0:
        interval_days = 1
    elif repetitions == 1:
        interval_days = 6
    else:
        match grade:
            1 (Hard):  interval_days = round(interval_days * HARD_INTERVAL_MULT)
            2 (Good):  interval_days = round(interval_days * ease_factor)
            3 (Easy):  interval_days = round(interval_days * ease_factor * EASY_BONUS)
    repetitions += 1
    match grade:
        1 (Hard):  ease_factor -= 0.15
        2 (Good):  unchanged
        3 (Easy):  ease_factor += 0.15

ease_factor = max(EASE_FLOOR, ease_factor)
interval_days = max(1, interval_days)   // belt-and-suspenders for tiny rounding edge cases
due_date = today + interval_days
```

`max(1, …)` on the post-rounding interval is a guard. Mathematically the formula can't produce 0 (smallest case is `round(1 * 1.2) = 1`), but it costs nothing to enforce the table-level `CHECK (interval_days >= 1)`.

### Streak update (also on review submission)
Run within the same transaction as the SM-2 update.
```
if last_practice_date is None:
    streak_count = 1
elif last_practice_date == today:
    no change
elif last_practice_date == today - 1 day:
    streak_count += 1
else:
    streak_count = 1
last_practice_date = today
```

### Implementation shape

```rust
pub struct ProgressState {
    pub ease_factor: f64,
    pub interval_days: i32,
    pub repetitions: i32,
    pub due_date: NaiveDate,
}

pub fn next_state(prev: ProgressState, grade: u8, today: NaiveDate) -> ProgressState;
```

Pure function, no I/O. The route handler reads the existing `user_case_progress` row (or constructs the default state if absent), calls `next_state`, then writes the result.

---

## 5. API surface — M3 additions

Prefix `/api/v1`. All require auth.

| Method | Endpoint | Body | Returns |
|--------|----------|------|---------|
| GET | `/study/due` | — | `{ cases: Case[], streak: { count, last_practice_date } }` — cases sorted oldest-due first |
| POST | `/study/:case_id/review` | `{ grade: 0..=3 }` | `{ case: Case, streak: { count, last_practice_date } }` — `case.state` reflects the post-review state |

The `Case` shape grows one field — `state: "not_started" | "learning" | "due" | "mastered"` — and that's the only change to existing endpoints.

### `POST /study/:case_id/review` semantics

- If no `user_case_progress` row exists for `(user_id, case_id)`, **the review creates one** with default state (`ease=2.5, interval=1, reps=0, due=today`) and immediately applies the grade. This is the "first review creates the row" behavior from `Cube_Practice_Design_Doc.md` §4 — no separate "start" endpoint.
- Validation: `grade` must be 0, 1, 2, or 3; `case_id` must reference a real case (otherwise 404).
- Wrapped in a transaction: SM-2 upsert + streak update on `users` are atomic.
- Returns the merged `Case` (post-review) plus the updated streak object so the frontend can refresh both header and queue.

### Streak count on `/auth/me`?
Not added. `/auth/me` stays identity-only per design doc §6. Streak rides on `/study/due` and `/study/:case_id/review` responses; M4's `/progress` will surface it as a primary stat.

---

## 6. Frontend — M3

### Routes
| Path | Auth | Notes |
|------|:---:|------|
| `/` (Practice tab inside shell) | required | real view replacing `PracticeStubView` |
| `/study` | required | full-bleed session screen — no tab bar |
| `/study?case=<uuid>` | required | same view, single-card session for the named case |

`/study` is a top-level route (not nested under the AppShell) so the session takes over the screen entirely.

### Components
- **`PracticeView.vue`** — renders inside the AppShell. Top: streak count (e.g. `🔥 7-day streak` — text only, no emoji unless approved). Below: due-card count + "Start session" button. Empty state when due == 0 nudges toward Cases. Calls `studyStore.loadDue()` on mount.
- **`StudySessionView.vue`** — full-bleed. Top bar with "× End session" + progress dots (one per queue position, colored by past grade). Card body: case header eyebrow, big pattern (size 240), "Reveal answer" button → reveals algorithm + result preview + 4 grade buttons. On grade tap, posts the review and advances. After last card, shows session-complete summary with per-grade counts.
- **`SessionCompleteView.vue`** (composed inline inside StudySessionView, not a separate route) — total cases, grade tally, "Back to practice" button.
- **`CaseStatePip.vue`** — small reusable indicator. Color/shape per state:
  - `not_started`: hollow circle, faint
  - `learning`: half-filled, accent
  - `due`: solid, warning color
  - `mastered`: solid checkmark, success color

### Pinia: `studyStore`
```ts
state: {
  queue: Case[]              // due cases to walk through
  index: number              // current position in queue
  results: Array<{ caseId: string, grade: 0|1|2|3 }>
  status: 'idle' | 'loading' | 'in_session' | 'complete' | 'error'
  streak: { count: number, last_practice_date: string | null } | null
  error: string | null
}
actions: {
  loadDue()                  // GET /study/due, populates queue + streak
  startSingle(caseId)        // builds a 1-element queue from casesStore.byId
  submitGrade(grade)         // POST /study/:case_id/review, advances index
  endSession()               // resets queue + index + results
}
getters: {
  currentCase()              // queue[index] or null when complete
  remainingCount()
}
```

The store is in-memory only. Reloading the page abandons the session (acceptable trade-off — sessions are typically <2 min).

### "Start studying" on case detail
- `CaseDetailView` reads `state` off the merged case.
- If `state === 'not_started'`, show a primary "Start studying" button.
- Clicking calls `studyStore.startSingle(caseId)` then `router.push('/study?case=<id>')`.
- The session then runs as normal — first grade submission creates the progress row server-side via the implicit-create review behavior.

### Streak / due count refresh
After every grade submission, the response carries the updated streak. `studyStore` writes both back, and `PracticeView` reads them via the store.

---

## 7. Security notes specific to M3
- Both new endpoints are auth-gated.
- `POST /study/:case_id/review` writes to `(user_id, case_id)` always bound to the authenticated `user_id` — no path for one user to write into another's progress.
- Grade validated to `0..=3`; out-of-range hits the `validation` envelope.
- Streak update is in the same transaction as the SM-2 write — no half-tick if SM-2 fails.

---

## 8. Testing strategy

**Backend (cargo test):**
- `srs` unit tests — exhaustive grid of `next_state` cases. Must cover:
  - Fail at rep 0 (already 0 → stays 0; interval to 1; ease −0.20)
  - Fail at rep > 0 (resets reps to 0; interval to 1; ease −0.20)
  - Good at rep 0 → rep 1, interval 1
  - Good at rep 1 → rep 2, interval 6
  - Good at rep 2+ → interval *= ease, ease unchanged
  - Hard at rep 2+ → interval *= 1.2, ease −0.15
  - Easy at rep 2+ → interval *= ease * 1.3, ease +0.15
  - Ease floor: starting at 1.3, Fail does not push below 1.3
  - Round-tripping through several reviews matches a hand-computed sequence
- `study::due_for_user` integration tests:
  - empty list when no progress rows
  - returns only rows where `due_date <= today`
  - rows from other users don't leak
  - sort order: oldest due first
- `study::apply_review` integration tests:
  - first review creates the row
  - second review updates the row (no duplicate)
  - streak ticks on first-ever review (None → 1)
  - streak holds at 1 if reviewing twice on the same day
  - streak ticks +1 when last_practice_date is yesterday
  - streak resets to 1 when last_practice_date is older than yesterday
  - 404 on unknown `case_id`
  - validation error on grade=4
- `cases::merge` integration tests gain coverage for the new `state` field — `not_started` for missing row, `due`/`learning`/`mastered` for varying interval/due_date.

**Frontend (Vitest):**
- `studyStore` actions with mocked api: `loadDue` populates queue, `submitGrade` advances index and writes streak, `endSession` clears state.

**Manual QA:** §10 below.

---

## 9. Configuration / environment

No new env vars. SM-2 constants live in `backend/src/srs/mod.rs` as `pub const` — promotable to env-tunable later if we want per-instance tuning, but for MVP they're hardcoded per the design doc.

---

## 10. "Done when" checklist (run on the deployed instance)

A literal QA script. M3 is closed when every line passes against `https://cube.nebulouscode.com` hitting `https://api.cube.nebulouscode.com`.

- [ ] Fresh account: Practice tab shows "Nothing due" empty state with a nudge toward Cases. Streak shows 0.
- [ ] Cases tab: every tile shows a state pip. All 57 read `not_started` for the fresh account.
- [ ] Open any case: detail view shows a `not_started` state pip and a "Start studying" button. Algorithm/result preview render normally.
- [ ] Tap "Start studying" → routes to `/study` with that one case in the queue. Pattern at large size, "Reveal answer" button.
- [ ] Tap "Reveal answer" → algorithm + result preview + 4 grade buttons appear.
- [ ] Tap "Good" → progress dot fills, session-complete view appears (1 of 1).
- [ ] Back to Practice tab: streak shows 1; due count is 0 (we just reviewed it; due_date moved forward).
- [ ] Cases tab: that case's tile pip is now `learning` (interval=1d, due tomorrow).
- [ ] Wait until tomorrow (or hand-edit `due_date` in psql for testing): the same case shows up as due. Streak is 1.
- [ ] Practice tab: "Start session" walks the case. Submit "Good" → streak ticks to 2.
- [ ] Submit a Fail on a card with rep > 1: card resets to rep 0, interval 1, ease drops by 0.20. Verify via psql.
- [ ] After 5+ Good reviews on a single card: state pip transitions to `mastered` once interval crosses 21 days.
- [ ] Direct nav to `/study` while signed out → redirects to `/login?next=/study`.
- [ ] Two browsers signed in as different users: user1 starts a case; user2's view of the same case is still `not_started`.
- [ ] End-session via the × button mid-queue exits cleanly without recording the un-graded current card.
- [ ] Backend SQL spot-check: `SELECT count(*) FROM user_case_progress` matches the number of distinct (user, case) pairs you've reviewed.

---

## 11. Story list

Pairs backend + frontend per the user's principle (avoid landing endpoints with no UI).

### Schema
- [x] **B1.** Migration `0004_user_case_progress.sql` — table, constraints, index, trigger reuse. 7 schema integration tests in `backend/tests/progress_schema.rs` cover existence, negative-interval/repetition/grade rejection, unique-pair, cascade-on-user-delete, and Anki initial defaults.

### SM-2 module (pure)
- [x] **B2.** `backend/src/srs/mod.rs` — `ProgressState`, `Grade` enum, `next_state(prev, grade, today)`, public constants. 13 unit tests cover Fail at rep 0/rep>0, Fail with ease floor, Good at each rep stage, Hard interval mult + ease drop, Easy bonus + ease rise, ease floor across consecutive Hards, multi-review round trip, and `Grade::from_u8` round trip + rejection.

### Backend study module (paired with frontend below)
- [x] **B3.** `cases::merge` extension — added `state` field with `CASE WHEN ... ELSE 'mastered'` derivation in the LEFT JOIN to `user_case_progress`. New `CaseState` enum exposed publicly. 3 new tests in `cases_merge.rs` cover Due, Learning, and Mastered (the existing 57-list test covers NotStarted).
- [x] **C1.** `study::due_for_user` + `GET /study/due`. Returns merged cases where state='due', sorted by oldest-due first, plus user streak. Tests for empty list, filtering by state, oldest-first sort, cross-user isolation. New `cases::list_due_for_user` helper.
- [x] **C2.** `study::apply_review` + `POST /study/:case_id/review`. Reads or defaults the progress row, calls `srs::next_state`, upserts, ticks streak — all in one transaction. Validates grade ∈ 0..=3 (Validation envelope on out-of-range), 404 on unknown case. Returns merged `Case` + `Streak`. 8 integration tests cover: first-review-creates-row, second-review-updates-row, same-day streak holds, streak ticks on yesterday-last-practice, streak resets on 2-day gap, unknown case 404, fail-resets-card, cross-user isolation.

### Frontend
- [x] **D1.** `studyStore` Pinia store. State: `queue`, `index`, `results`, `status`, `streak`, `error`. Actions: `loadDue`, `startSingle(caseId)`, `startSession`, `submitGrade(grade)`, `endSession`, `$reset`. `currentCase` and `remaining` getters. `submitGrade` posts the review and replaces the cached row in `casesStore` with the post-review state. Wired into `auth.ts`'s `resetUserScopedStores()`.
- [x] **D2.** `CaseStatePip.vue` — small SVG indicator with 4 visual states (hollow circle, half-filled, solid dot, solid + checkmark). Tooltip + optional inline label. Integrated into `CasesView` tiles (top-right corner overlay) and `CaseDetailView` title row.
- [x] **D3.** `PracticeView.vue` replaces `PracticeStubView`. Header with greeting + streak count, "Due today" card with count + Start-session button, empty state nudging toward Cases when due is 0. Calls `studyStore.loadDue()` on mount. `PracticeStubView.vue` deleted.
- [x] **D4.** `StudySessionView.vue` at `/study` — top bar with × End session + position label, per-card progress dots, big pattern (240px), "Reveal answer" gate, then algorithm + result preview + 4 grade buttons (Fail/Hard/Good/Easy with the prototype's tinted on-paper palette). Auto-advances on grade. Session-complete view with grade tally and qualitative summary line. Routes back to `/` on End/Done.
- [x] **D5.** `CaseDetailView` "Start studying" wiring. Shows `CaseStatePip` with label in the title row. Adds primary-CTA button: "Start studying" when `state === 'not_started'`, "Practice now" when `state === 'due'`. Both call `studyStore.startSingle()` and route to `/study`.

### QA
- [ ] **E1.** Walk §10 on the deployed instance. Update notes/TODOs with anything that surfaces.

---

## 12. Notes / open items

- **Timezone for "today".** Streak rollover and due-date comparisons need a notion of "today." Simplest: server UTC date. A user in PST rolling at 4 PM their time would see streaks tick at 5 PM Pacific (00:00 UTC). For MVP that's tolerable; post-MVP we can carry `users.timezone` and roll per-user. *Default: UTC. Not surfaced in the UI.*
> Response: Sounds good. Roll over should happen at midnight local time.

- **End-session mid-queue: discard the current card?** Yes — only reviewed cards are submitted. The current un-graded card stays at its previous state. Confirm.
> Response: Sounds good

- **Grade-button colors / labels.** The prototype's RATINGS palette is muted (washes of red/yellow/green/blue at low alpha). Worth porting verbatim, or do we want the buttons styled paper-neutral with text-only labels? *Default: port the prototype's tinted backgrounds.*
> Response: yes I liked those colors. The wash out on the paper suits me

- **Reveal flow: tap-to-reveal vs auto-reveal.** Prototype uses an explicit "Reveal answer" button — the user is supposed to mentally execute the algorithm before checking. Confirm we keep this gate (most people doing OLL drills want to test recall first).
> Response: yes tap/click to reveal

- **First-review-creates-row vs explicit /study/:id/start.** Going with implicit create per design doc §4. Means there's no dedicated "Start studying" API call — the first POST `/review` does double duty. Frontend "Start studying" button is just a navigation shortcut to the session screen. Confirm.
> Response: yes that makes sense we don't need an endpoint for every button.

- **Streak resilience to clock skew.** If a user reviews near midnight UTC, two reviews 5 minutes apart can fall on different "today" values. Acceptable edge case.
> Response: I can live wit that. Review roll over date and user timezone can be added together later. Make a note of these in the main design doc as post MVP tasks

- **Visible streak when 0.** Display "0" or a subtle "Start a streak" prompt? *Default: show the count, even when 0.*
> Response: Show 0


---

## 13. Looking back

A retrospective pass added after the milestone shipped. Fill in honestly —
the value of these notes is the friction they capture, not a victory lap.

- **What I'd do differently if I started this milestone today:**

- **Surprises during execution:**

- **Decisions that turned out to matter more than expected:**

- **Decisions that turned out not to matter:**

- **What this milestone taught me that I'd carry into future work:**
