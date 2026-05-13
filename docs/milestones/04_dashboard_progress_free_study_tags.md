# Milestone 4 — Dashboard, Progress, Free Study, Tags

Detailed design + story list for M4. Scope set in `docs/milestones/README.md`. Authoritative spec is `docs/Cube_Practice_Design_Doc.md` §1 (free study + dashboard goals), §3 (schema), and the M2 follow-through on the Tier-2 → multi-tag rework recorded in `docs/outstanding_decision.md`.

---

## 1. Goal recap

By the end of M4:

- The Practice tab is a real dashboard: streak, due-today, learning/mastered counts, plus a "Today's queue" CTA. The "Standing" card from `initial_design/src/screen-home.jsx` lands in a simplified SM-2-native form (no letter grades — distribution by `not_started` / `learning` / `due` / `mastered`).
- `/progress` replaces `ProgressStubView.vue` with a real per-case breakdown: filter chips for the four states + a flat list of cases sorted by case number, each row showing the same `CaseStatePip` + state label.
- Free study works: the Cases view (or a new entry on Practice) lets the user start a study session over an arbitrary filtered subset (primary shape, tags, status) — not just due cards.
- The single-string `tier2_tag` collapses into a multi-valued `tags TEXT[]` on `cases` and `user_case_settings`. Backfill the existing single value as a one-element array. Merge SQL combines arrays. Detail view input is comma-separated; cases browser switches from grouped-by-Tier-2 to a flat list with chip-based any-of tag filtering.
- A "Stats over time" placeholder panel appears at the bottom of `/progress` saying "Coming soon — review history charts."

Out of scope (deferred):
- Real charts on the stats panel. The post-MVP list (`Cube_Practice_Design_Doc.md` §1) owns this.
- Polish, empty-state design, onboarding — M5.
- Guest mode — M6.
- The `users.timezone` + per-user midnight rollover — post-MVP, captured in the design doc.

---

## 2. Architecture for M4

### Backend
- One migration converting `cases.tier2_tag TEXT` and `user_case_settings.tier2_tag TEXT` into `tags TEXT[]`. Backfills each non-null `tier2_tag` as a one-element array, drops the old columns. The merge SQL becomes `COALESCE(ucs.tags, c.tags)` (override-replace, mirroring every other settings field).
- Update `cases::Case`, `cases::SettingsPatch`, the `update_settings` resolver, and `MERGE_SELECT` to carry `tags: Vec<String>`.
- New module `backend/src/progress/mod.rs`. Single function `summary_for_user(pool, user_id) -> ProgressSummary` — totals per state + streak. Cheap aggregation over the merged-cases query (or a direct `GROUP BY` on `user_case_progress` joined with `cases`).
- New routes module `backend/src/routes/progress.rs`: `GET /progress` (summary) and `GET /progress/cases?state=…` (filtered list). The latter is a thin wrapper on `cases::list_for_user` with an optional state filter pushed into SQL.
- No new endpoint for free study. Free study reuses `/cases` (already returns every merged case) and the existing `POST /study/:case_id/review` per-card path. The session queue is built client-side from the filtered list.

### Frontend
- **`PracticeView.vue`** grows the "Standing" card below the existing KPI row. Streak + Due stay where they are. Standing card shows the four state counts as a small distribution bar + chip row (similar to the prototype's grade distribution but with our four states).
- **`/progress` route** — new `ProgressView.vue` replacing `ProgressStubView`. Header, summary tiles (state counts), state filter chips, flat per-case list (each row: case number + name + tier1 chip + state pip + grade-history dot row). Bottom: "Stats over time — coming soon" card.
- **`/free-study` route** (new, full-bleed setup screen) — filter UI: primary shape chips, tag chip multi-select, state checkboxes. Live preview count "N cases match." "Begin session →" builds the queue and pushes to `/study` (the existing study session view, no changes needed there).
- **`CasesView.vue`** rework — drop the grouped-by-tier2 layout. Flat list sorted by case number with two filter rows: tier1 chips (existing) + tag chips (new, multi-select, any-of). The search box stays. The grouped headers are gone.
- **`CaseDetailView.vue`** — replace single-tag input with comma-separated input. Display tags as chips (read-mode) and as an `<input>` (edit-mode) per detail-view convention. The `TIER2_LABELS` mapping disappears — tags render as raw strings.
- **`progressStore`** Pinia store — keeps the summary fresh (after each review submission, dirty + lazy reload).

### Reuse from M1/M2/M3
- `StudySessionView.vue` — unchanged. Free study queues feed into the same view.
- `studyStore.startSession` — extend to accept a custom queue (currently builds from due list).
- `casesStore` — already client-side cached. Free-study filter UI reads from it directly.
- `CaseStatePip.vue`, `PatternDiagram.vue`.

---

## 3. Schema — M4 changes

### Migration `0005_tags_array.sql`

```sql
-- Convert single-string tier2_tag → tags TEXT[].
ALTER TABLE cases
    ADD COLUMN tags TEXT[] NOT NULL DEFAULT '{}';

UPDATE cases
   SET tags = ARRAY[tier2_tag]
 WHERE tier2_tag IS NOT NULL;

ALTER TABLE cases DROP COLUMN tier2_tag;

ALTER TABLE user_case_settings
    ADD COLUMN tags TEXT[];

UPDATE user_case_settings
   SET tags = ARRAY[tier2_tag]
 WHERE tier2_tag IS NOT NULL;

ALTER TABLE user_case_settings DROP COLUMN tier2_tag;

-- GIN index for the eventual "filter cases by any-of tags" query in
-- free-study and the cases browser. Cheap to add now.
CREATE INDEX cases_tags_gin_idx ON cases USING GIN (tags);
```

Notes:
- `cases.tags` is `NOT NULL DEFAULT '{}'`. A case with no tags is an empty array, never NULL — simplifies all downstream code.
- `user_case_settings.tags` stays nullable. NULL = "no override, use global." Per §12 #1+#2, an empty user-tags input deletes the override (stores NULL) — there is no "explicit empty override" state. The resolver treats `Some(Some(vec![]))` the same as `Some(None)`.
- When set, the user's tag array fully **replaces** the global set. Mirrors every other override column.

### Normalization rules

Applied on every write (PATCH `/cases/:id/settings`):

1. Trim whitespace on each tag.
2. Lowercase ASCII letters (`s.to_ascii_lowercase()` — leave non-ASCII alone for now).
3. Drop empty strings.
4. Dedupe (preserve first-seen order).
5. Reject any tag longer than 60 chars (validation envelope).

Applied via a single `cases::normalize_tags(Vec<String>) -> Vec<String>` helper.

### State derivation — unchanged

The four-state derivation from M3 §3 stays exactly as it is. M4 adds endpoints that aggregate over it.

---

## 4. API surface — M4 additions and changes

Prefix `/api/v1`. All require auth.

### New

| Method | Endpoint | Body | Returns |
|--------|----------|------|---------|
| GET | `/progress` | — | `{ summary: { not_started, learning, due, mastered }, total: 57, streak: { count, last_practice_date } }` |
| GET | `/progress/cases?state=<state>` | — | `{ cases: Case[] }` — filtered by SM-2 state, falls back to all when omitted |

`state` is one of `not_started`, `learning`, `due`, `mastered`. Validation envelope on bad values.

### Changed shape

- The `Case` JSON shape gets `tags: string[]` and **loses** `tier2_tag`. This is a breaking change for anything caching the old shape — only the frontend, and only since M2. Search/replace.
- `PATCH /cases/:id/settings` — body field `tier2_tag: Option<Option<String>>` becomes `tags: Option<Option<Vec<String>>>`. `null` (or empty `[]`, which the resolver coerces to `null`) clears the override and falls back to global.

No other M2 or M3 endpoint changes.

---

## 5. Frontend — M4

### Routes

| Path | Auth | Notes |
|------|:---:|------|
| `/` (Practice tab) | required | Gains "Standing" card |
| `/progress` (Progress tab inside shell) | required | Replaces `ProgressStubView` |
| `/free-study` | required | Full-bleed setup screen — no tab bar |

`/free-study` is full-bleed because it's a setup screen for a study session — same posture as `/study`.

### Views

- **`PracticeView.vue` (extended)** — Adds the Standing card below the queue CTA. Layout: small distribution bar (4 segments colored by state) + 4 chips showing the count per state. No letter grades, no scores. Source: `progressStore.summary`. On mount, calls `progressStore.ensureLoaded()` alongside the existing `studyStore.loadDue()`.

- **`ProgressView.vue` (new)** — Header with eyebrow + serif title ("Where you stand"). Summary chip row showing the four state counts (clickable filters). List of cases matching the active filter, each row:
  - Case number badge ("01" — paper-card)
  - Name (nickname or default), italic serif
  - `CaseStatePip` + textual state
  - Tier-1 dot in muted color
  - Tappable → routes to `/cases/:id`
  - At the bottom, a "Stats over time" card with light-italic "Charts and trends — coming soon" copy. Sized to match the rest of the page.

- **`FreeStudyView.vue` (new)** — Full-bleed setup screen. Three filter sections:
  - **Primary shape** — chip row, same five chips as the Cases browser, single-select (`all`/`*`/`L`/`-`/`+`)
  - **Tags** — chip row built from the union of all tags currently present on any merged case. Multi-select, any-of semantics.
  - **State** — four checkboxes (`not_started`, `learning`, `due`, `mastered`). Default: all checked.
  - Live count: "N cases match." Disabled "Begin session →" button when N = 0.
  - On submit, calls `studyStore.startSession(filteredCases)` and routes to `/study`.

- **`CasesView.vue` (rework)** — Drop the grouped-by-tier2 layout. Replace with a flat list of tiles sorted by case number. Add a tags chip row above the list (any-of multi-select, populated from the case-store's `allTags` getter). Tier-1 chips and search bar stay where they are.

- **`CaseDetailView.vue` (rework)** — In read mode, render tags as a row of small paper chips (one per tag). In edit mode, render a single comma-separated `<input>` placeholder "needs work, fish, …". On save, parse, normalize, and send as `tags`. Drop the `TIER2_LABELS` mapping.

### Pinia: `progressStore` (new)

```ts
state: {
  summary: { not_started, learning, due, mastered, total } | null
  status: 'idle' | 'loading' | 'ready' | 'error'
  error: string | null
}
actions: {
  ensureLoaded()        // GET /progress, idempotent
  reload()              // force-refresh — call after a review submission
  $reset()
}
getters: {
  total()
  startedCount()        // learning + due + mastered
}
```

`studyStore.submitGrade` calls `progressStore.reload()` after a successful review so the dashboard tile counts stay accurate. Wired into `auth.ts`'s `resetUserScopedStores()`.

### Pinia: `casesStore` (extension)

Add `allTags: ComputedRef<string[]>` — sorted unique tags across the merged list, used by Cases browser + Free Study. Drop `groupedByTier2`.

### Pinia: `studyStore` (extension)

`startSession(queue?: Case[])` — if `queue` is provided, use it verbatim; otherwise build from the current due list (existing behavior). No other changes; the session view doesn't care where the queue came from.

---

## 6. Security notes specific to M4
- New endpoints are auth-gated.
- `GET /progress` and `GET /progress/cases` only ever read rows scoped to the authenticated `user_id` — no path for cross-user leakage.
- Tag normalization happens server-side. Client-side normalization is a UX nicety, not a security boundary.
- Tag length cap (60 chars) prevents pathological inputs. No HTML escape needed since tags render as text inside Vue templates.

---

## 7. Testing strategy

**Backend (cargo test):**
- `cases::normalize_tags` unit tests — empty input, mixed case, leading/trailing whitespace, duplicates, length cap rejection, non-ASCII passthrough.
- Migration test: existing `tier2_tag = 'fish'` becomes `tags = ['fish']`; NULL becomes `[]` for cases / NULL for user_case_settings.
- `cases::merge` integration tests gain coverage for `tags` override semantics: NULL ucs → global tags, `[]` ucs → empty override, non-empty ucs → replaces global.
- `update_settings` integration tests for `tags` patch:
  - `Some(Some(["fish","needs work"]))` writes the override.
  - `Some(None)` (null in JSON) clears the override (NULL — falls back to global).
  - `Some(Some([]))` is coerced to `Some(None)` and clears the override.
  - Normalization applied on write (mixed-case input lands lowercased).
  - Length cap rejection lands in validation envelope.
- `progress::summary_for_user` integration tests:
  - Fresh user → all 57 in `not_started`.
  - Mix of states → counts add up to 57.
  - Cross-user isolation.
  - Streak rides on the response.
- `progress/cases?state=` integration tests — filtering correctness, validation on bad state.

**Frontend (Vitest):**
- `progressStore.ensureLoaded` populates summary; `reload` clears + refetches.
- `casesStore.allTags` returns sorted unique union across the merged list.
- Tag input parser: comma-split, trim, dedupe, lowercase round-trip.

**Manual QA:** §10 below.

---

## 8. Configuration / environment

No new env vars.

---

## 9. Migration / data risk

The `tier2_tag` → `tags TEXT[]` migration is the only schema risk in M4. Mitigations:

1. Migration runs in one transaction (sqlx default). On failure, rollback leaves the column intact.
2. The seeded global tags are a single value per case (the existing `tier2_tag` mapping), so the backfill is deterministic and trivially verifiable: `SELECT count(*) FROM cases WHERE array_length(tags, 1) = 1` should equal the count of pre-migration cases with non-null `tier2_tag`.
3. User overrides at this stage are still rare (M2 just landed). Worst case is one user's single-tag override is migrated as a one-element array — no data loss possible.

The frontend ships in lockstep — old client + new server still works for `GET /cases` (Vue ignores the new field, but reading the old `tier2_tag` would 500). We take the deploy-coordinated breakage as acceptable since there are no production users yet.

---

## 10. "Done when" checklist (run on the deployed instance)

A literal QA script. M4 is closed when every line passes against `https://cube.nebulouscode.com` hitting `https://api.cube.nebulouscode.com`.

- [ ] Cases tab shows a flat list (no group headers) sorted by case number.
- [ ] Tier-1 chip filter still works.
- [ ] New tag chip row above the list — selecting any subset narrows the list to cases with at least one of those tags.
- [ ] Search box still matches against case numbers, nicknames, algorithms, and tags.
- [ ] Open a case detail: tags render as comma-separated chips in read mode.
- [ ] Tap edit: tags become a comma-separated input. Add a free-form tag like `needs work` and save. The chip appears.
- [ ] Save edit with empty tag input: tags clear (override = empty array). Browser tile shows no tags.
- [ ] Practice tab: Standing card visible below the queue. Counts add up to 57.
- [ ] Progress tab opens a real view. State chip row at top. Tap "due" → list filters to due cases.
- [ ] Bottom of progress: "Stats over time — coming soon" placeholder.
- [ ] Free study (entry from… see §12 open item): primary shape chip + tag chips + state checkboxes. Live "N cases match" count updates as filters change.
- [ ] Free study: pick "all" + "fish" tag + "not_started" state → "Begin session →" walks only matching cases.
- [ ] Mid-session, fail/grade behaviour identical to M3.
- [ ] After session, dashboard counts and progress view both reflect the new state without a hard refresh (relies on `progressStore.reload()`).
- [ ] Two browsers signed in as different users: user1's tag override on case 02 doesn't appear in user2's view (still shows global tags).
- [ ] psql spot-check: `SELECT case_number, tags FROM cases ORDER BY case_number LIMIT 5` returns array values, not single strings.

---

## 11. Story list

Pairs backend + frontend per the user's principle (avoid landing endpoints with no UI).

### Schema + tag rework
- [ ] **B1.** Migration `0005_tags_array.sql` — add `tags TEXT[]`, backfill from `tier2_tag`, drop the old columns, add GIN index. Test: existing single-tag rows round-trip, NULL rows become empty array on cases / NULL on user_case_settings.
- [ ] **B2.** `cases::normalize_tags` helper + 6 unit tests. Wire into `update_settings`. Update `Case` struct + `MERGE_SELECT` + `SettingsPatch` to carry `tags: Vec<String>`. Update existing `cases_merge` integration tests for the new shape.
- [ ] **D1.** Frontend tag rework, paired with B1+B2. `Case.tags: string[]` on the store; `casesStore.allTags` getter; drop `groupedByTier2` and `TIER2_LABELS`. Update `CasesView` to flat list + new tag chip row. Update `CaseDetailView` to chip read + comma-separated edit. Verify GET/PATCH round-trip in the deployed app before opening B3.

### Progress endpoint + view
- [ ] **B3.** `progress::summary_for_user` + `GET /progress`. Returns the 4 state counts + total + streak. Tests cover fresh user (all 57 not_started), mixed states, and cross-user isolation.
- [ ] **B4.** `GET /progress/cases?state=…` — wraps `cases::list_for_user` with a SQL filter. Validates state. Tests cover each filter value + bad value rejection.
- [ ] **D2.** `progressStore` Pinia store. State + actions per §5. Wire into `auth.ts` resetUserScopedStores. Call `progressStore.reload()` from `studyStore.submitGrade`.
- [ ] **D3.** `ProgressView.vue` replaces `ProgressStubView.vue`. Header, state chip filter, flat case list with state pip + click-through to detail, "Stats over time — coming soon" card at bottom. Calls `progressStore.ensureLoaded()` on mount.

### Dashboard polish
- [ ] **D4.** `PracticeView.vue` grows a "Standing" card below the queue card. 4-segment distribution bar + 4 small count chips. Reads `progressStore.summary`. No backend change.

### Free study
- [ ] **D5.** `studyStore.startSession(queue?)` extension — accepts a custom queue. Backwards-compatible default uses the due list.
- [ ] **D6.** `FreeStudyView.vue` at `/free-study` (full-bleed). Filter UI + live count + "Begin session →" that builds the queue from `casesStore.list` filtered client-side. Routes back to wherever the user came from (Practice / Cases) on cancel.
- [ ] **D7.** Wire entry points: button on `PracticeView` ("Free study →" link below the Standing card) and a "Free study" button at the top of `CasesView`. Two entries, one view.

### QA
- [ ] **E1.** Walk §10 on the deployed instance. Update notes/TODOs with anything that surfaces.

---

## 12. Notes / open items

- **Override semantics for `tags`: replace or union with global?** Default: replace (mirrors every other override field). A user who wants both the global tags + their own can re-type the global ones. *Confirm.*
> Response: User gets the global by default but has the option to delete it.

- **Empty array as override: explicit no-tags vs reset?** Default: NULL (`Some(None)` from the patch) means "clear the override, use global"; empty array (`Some(Some([]))`) means "I want this case to have zero tags." *Confirm.*
> Response: That works. User can delete the existing tag which should save null/None to db.

- **Tag normalization: lowercase ASCII only?** Default: lowercase ASCII letters, leave Unicode alone. Means "Fish" and "fish" dedupe but "Δ" stays "Δ". *Confirm — alternative is a strict whitelist that drops non-ASCII entirely.*
> Response: that works

- **Free study entry points.** Two candidates: a "Free study" button on PracticeView (mockup-ish) and one on CasesView (fits the "browse → practice subset" mental model). Default: ship both. *Confirm.*
> Response: Doing both works, match the mockup, I think it had both.

- **"Stats over time" placeholder copy.** Default: "Charts and trends — coming soon." Goes in a `<Card>` at the bottom of `/progress`. *Open to a different headline.*
> Response: Sounds good.

- **Standing card on Practice: order of states.** Default left-to-right: `mastered` (success), `learning` (accent), `due` (warning), `not_started` (faint). Reads as "best to worst" so a user with mostly green sees green first. *Confirm or flip.*
> Response: Match the mockup please, if this isn't defined then you're fine to go with this.

- **Tag length cap.** Default 60 chars per tag. Long enough for "needs work after the jaw" and short enough that nobody pastes a paragraph in by accident. *Confirm.*
> Response: works for me

- **Progress tab: list inside vs scoped page.** The Cases tab and Progress tab will both end up showing every case, just sorted/filtered differently. Default: keep them separate — Cases is browse + edit (no state filter), Progress is "what's my training look like" (state-first). *Confirm — alternative is to fold state filtering into Cases and drop the Progress tab to a stats-only page.*
> Response: that's fine to show both twise, it's different sort which is helpful.

---

## 13. Looking back

A retrospective pass added after the milestone shipped. Fill in honestly —
the value of these notes is the friction they capture, not a victory lap.

- **What I'd do differently if I started this milestone today:**

- **Surprises during execution:**

- **Decisions that turned out to matter more than expected:**

- **Decisions that turned out not to matter:**

- **What this milestone taught me that I'd carry into future work:**
