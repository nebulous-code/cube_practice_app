# Outstanding Decisions

A running list of open questions and conflicts that need to be resolved before Vue/Rust implementation begins. Items are grouped roughly by impact — design conflicts first, then unspecified MVP details, then tooling choices.

---

## 1. Direct Conflicts Between Design Doc and Initial React Design

These are places where the spec (`OLL_App_Design_Doc.md`) and the React prototype (`initial_design/src/`) disagree. Each needs an explicit pick before Vue work starts.

### 1.1 Grading scale: 0–5 (SM-2 classic) vs 0–3 (Anki-style)
- **Spec:** SM-2 with grades 0–5 (`docs/OLL_App_Design_Doc.md:163-184`).
- **Prototype:** four buttons — Fail / Hard / Good / Easy (`srs.jsx:6-11`), values 0–3.
- **Why it matters:** SM-2's published formula assumes 0–5. A 4-button UI is friendlier but requires either remapping (e.g. 0→0, 1→2, 2→4, 3→5) or replacing SM-2's ease-factor formula with the prototype's recency-weighted score. Pick one and document the mapping if applicable.
> Response: let's map 0&1 to button 0, 2 & 3 to button 1, and 4 to button 2, and 5 to button 3


### 1.2 Scheduling model: SM-2 ease/interval vs recency-weighted score
- **Spec:** Per-user-per-case row with `ease_factor`, `interval_days`, `repetitions`, `due_date`.
- **Prototype:** Stores raw rating history per case and computes a 0–100 score on the fly with exponential recency weighting (`srs.jsx:15-30`). No real `due_date` — "due" is just `max(weakCount, 8)` (`screen-home.jsx:36-37`).
- These are fundamentally different mental models. SM-2 says "this card is due on date X"; the prototype says "your composite skill on this card is 62/100." Confirm we're going with SM-2 as the spec describes, and that the letter-grade UI (next item) is a *display layer* on top of SM-2 data — not a replacement for it.
> Response: TODO:

### 1.3 Progress states: A/B/C/D/F vs not_started/learning/due/mastered
- **Spec:** Four states for filtering and dashboard counts (`OLL_App_Design_Doc.md:268-272`, `:278`).
- **Prototype:** Six letter-grade buckets (A/B/C/D/F/New) computed from score (`srs.jsx:32-48`).
- **Decision needed:** Keep the spec's four-state taxonomy and drop letter grades, *or* keep letter grades as a richer display and define how they map to the four states. Also need explicit thresholds — at what `interval_days` / `repetitions` / `ease_factor` does a card become "mastered" vs "learning"?
> Response: TODO:

### 1.4 Diagrams: pre-built static SVGs vs dynamic pattern rendering
- **Spec:** 57 pre-built SVG files named `oll_{case_number:02}.svg`, served as static assets, rotated via CSS `transform` for back-of-card. "Dynamic diagram rendering is explicitly out of scope for MVP" (`OLL_App_Design_Doc.md:336-354`).
- **Prototype:** `<PatternDiagram>` renders dynamically from the 9-char pattern string (`diagram.jsx`). No SVG files exist on disk yet.
- **Decision needed:** (a) Generate the 57 static SVGs (from what source — the prototype's pattern strings? hand-drawn?), or (b) accept that the prototype's dynamic renderer is the implementation and update the spec accordingly. The dynamic approach is honestly simpler to maintain and already works — the spec's static-SVG decision should probably be revisited.
> Response: Keep the prototype's dnamic rendering incase we want to restyle it.

### 1.5 Information architecture: 3 bottom tabs vs 12 routes
- **Spec:** 12 named routes incl. `/login`, `/study`, `/study/free`, `/cases`, `/cases/:id`, `/progress`, `/settings`, etc. (`OLL_App_Design_Doc.md:307-324`).
- **Prototype:** Three bottom-tab views (Practice / Cases / Progress) with practice and case-detail as full-bleed modal-style routes (`app.jsx`). No login/settings UI at all.
- **Decision needed:** Confirm the final IA. Options:
  - Keep the prototype's 3-tab shell and add auth/settings as separate stacked routes outside the tab bar. Mobile-friendly.
  - Move to a more traditional Vue Router structure with 12 routes.
  - Hybrid: tab bar for the three core views, top-right menu for account/settings.
> Response: TODO: I need to understand this one better 

### 1.6 Streak tracking
- **Spec:** Listed under **Post-MVP** (`OLL_App_Design_Doc.md:26`).
- **Prototype:** Streak is a top-row KPI on the home screen with prominent treatment (`screen-home.jsx:65-72`).
- **Decision needed:** Cut from MVP and ship without it, or promote streak to MVP scope.
> Response: Promote to MVP scope

### 1.7 Stats / progress-over-time
- **Spec:** "Stats over time and progress graphs" listed under **Post-MVP** (`OLL_App_Design_Doc.md:24`).
- **Prototype:** Stats screen exists with weekly/30-day review counts and per-group breakdown (`screen-stats.jsx`).
- **Decision needed:** Same as above — does the existing stats screen ship in MVP, or get cut down to the simpler "progress" dashboard the spec calls for?
> Response: Cut it to be simpler. Leave the skeleton there so users know it's coming

### 1.8 Authentication in the prototype
- The prototype has no auth — it persists progress to `localStorage` (`app.jsx:5-16`).
- The spec mandates full auth in MVP (register, verify-email, login, password reset, JWT cookies).
- **Decision needed:** Confirm we are building the full auth flow as part of MVP from day one (implies Auth views need to be designed — see §2.1). Alternative is a "guest mode" that stores progress locally until the user signs up, but the spec doesn't mention this.
> Response: Confirmed, We are building the full auth flow as part of the MVP. That seemed trivial from a design standpoint so it wasn't added in the mockup

---

## 2. Things the Spec Mentions but Doesn't Fully Specify

### 2.1 Designs for auth-related views
The spec lists `LoginView`, `RegisterView`, `VerifyEmailView`, `ForgotPasswordView`, `ResetPasswordView`, `SettingsView` but no visual or interaction design exists for any of them. These need to be designed (or at least sketched) before frontend implementation.
> Response: These designs will be included shortly

### 2.2 Status thresholds
The four progress states (`not_started`, `learning`, `due`, `mastered`) are referenced in the API filter (`OLL_App_Design_Doc.md:272`) but never defined in concrete terms. Need explicit rules, e.g. "mastered = `repetitions >= 5 AND ease_factor >= 2.3`."
> Response: Need to discuss this more

### 2.3 Practice session size
The prototype hardcodes 10 (weakest) or 15 (all) cards per session (`app.jsx:28`). The spec doesn't specify. Decide: fixed size, all due cards, user-configurable, or capped-at-N-from-the-due-pile.
> Response: all due cards

### 2.4 Tag model — Tier 2 and user tags
- The spec has Tier 1 (`+`, `-`, `L`, `*`, "fixed, geometric"), Tier 2 (a single string per case, user-overridable), and free-form user-defined tags (many-to-many).
- The prototype only has `priority` (≈ Tier 1) and `group` (≈ Tier 2). No user-tag support.
- **Open questions:**
  - Can users invent new Tier 2 tags, or is the set fixed at the values used in `data.jsx` (T_shapes, fish, awkward_shape, …)?
  - Are user-defined free-form tags expected in MVP, or is this a place to defer to Post-MVP?
  - The spec says Tier 1 is "fixed, geometric" — not user-overridable. Confirm.
> Response: We'll enhance the prototype to let the user edit the tags.

### 2.5 Result rotation representation
- Spec: `result_rotation` is a string `NULL | "cw" | "180" | "ccw"` (`OLL_App_Design_Doc.md:71, 101`).
- Prototype: integer 0–3 quarter-turns CW (`data.jsx:3`).
- Pick one and use it consistently in DB schema, API JSON, and frontend.
> Response: I like integer best

### 2.6 Source of truth for the 57 OLL cases
The seed script needs canonical data for all 57 cases (algorithms, default nicknames, default Tier 2 tags, result mapping, diagram data). This data exists in `initial_design/src/data.jsx`. Confirm we'll port that as the seed source and that the values are correct (the user authored them, but spot-check before committing to migrations is wise).
> Response: They are correct

### 2.7 `diagram_data` JSONB shape
The spec keeps `diagram_data` in `cases` but says it's not used by MVP (`OLL_App_Design_Doc.md:353-354`). If we're going with static SVGs, do we still populate it? If we go with dynamic rendering (§1.4), this becomes the actual source of truth, and its schema needs nailing down — probably just the 9-char pattern string from the prototype.
> Response: yeah that works

### 2.8 Practice direction
The current prototype shows pattern → reveal algorithm → grade. Confirm this is the only direction (no reverse-recall mode where the user is shown the algorithm and must recall the pattern).
> Response: correct this study style has no point to reverse recall

### 2.9 First-time / empty-state UX
A user with zero reviews lands on the dashboard. What does it show? The prototype's home screen breaks slightly with no data. Need an empty-state design.
> Response: We can come up with this as we build. It will likely show 0 or No Data on first creation.

---

## 3. Tooling and Implementation Choices Not in the Spec

### 3.1 TypeScript
The spec says Vue 3 + Vite + Vue Router + Pinia. Doesn't specify TypeScript. Pick: TS or plain JS. (Recommend TS for an app this size — the case/progress shapes are non-trivial.)
> Response: TS

### 3.2 CSS strategy
The prototype uses massive inline-style objects (~every component). Vue idioms favor scoped `<style>` blocks. Need to pick:
- Plain `<style scoped>` per SFC
- CSS Modules
- Tailwind / UnoCSS
- A component library (Naive UI / PrimeVue / Vuetify) that brings its own styling

The prototype's paper/serif aesthetic should probably be preserved either way — confirm.
> Response: TODO: Understand this question better

### 3.3 Testing
Not mentioned. Pick: Vitest for unit, Playwright or Cypress for E2E. At minimum SM-2 should have unit tests.
> Vitest for unit tests. Backend is rust so it'll have its own unit tests

### 3.4 Linting / formatting
Not mentioned. Pick: ESLint + Prettier vs Biome.
> Response: No preference you choose

### 3.5 Backend Rust libraries beyond what's listed
The spec calls out Axum, sqlx, argon2, JWT, reqwest, tower-http. Other choices to make:
- Validator (e.g. `validator` crate) for request body validation?
- Error type strategy (`thiserror` + an `AppError` enum is conventional).
- `tracing` for logging? Spec mentions `RUST_LOG` so probably yes.
- DB connection pool size, statement timeouts.
> Response: All of that works for me

### 3.6 Light/dark theme
The prototype's `PatternDiagram` has a dark-mode option that's never toggled. Spec doesn't mention dark mode. Decide: support it in MVP, design later, or never.
> Light mode only for MVP

### 3.7 Responsive / desktop layout
Spec says "mobile-friendly web app." Prototype is mobile-only (fixed-feel narrow padding, bottom tab bar). Confirm: are we shipping a desktop layout, or is mobile-shaped-on-desktop acceptable for MVP?
> Response: mobile-shaped-on-desktop is desired for MVP. As a rule we are mobile first, even if we're not doing a mobile app yet.

---

## 4. Smaller Things to Confirm

- **JWT cookie SameSite:** Spec says `SameSite=Strict`. If frontend and backend live on different subdomains on Render (e.g. `app.x.com` and `api.x.com`), `Strict` will break the login redirect flow. May need `SameSite=Lax` or a same-origin reverse proxy.
    - that's fine we'll debug that as we go
- **Algorithm input format:** Will the app accept any free-form string in the algorithm field, or validate cube notation (`R U R'`, with `(x')`, lowercase wide moves, etc.)?
    - We'll allow freeform for now but that might be an enhancement later
- **57 cases UI ordering:** The prototype groups by shape group. Confirm grouping is the default browse order (vs case number, vs Tier 1).
    - Should sort by case number. TODO: Figure out if case number/name is universal
- **"Wrong? Edit →" affordance during practice:** Prototype offers an inline jump from the practice card to case-edit (`screen-practice.jsx:142-148`). Confirm that interrupting practice to edit a case is desired UX (vs flagging the case to review later).
    - That is desired
- **Cold-start on Render free tier:** Render free web services sleep after inactivity. First request after a sleep takes 30–60s. Acceptable for MVP, or do we need a paid tier from day one?
    - Cold start is acceptable

---

## Recommended next step

Triage §1 (the conflicts) first — pick the model for grading + scheduling + state taxonomy + diagrams + IA + auth scope, since each of those reshapes the rest. §2 and §3 can mostly be resolved as we build, but §1 decisions block the schema and the route structure.

