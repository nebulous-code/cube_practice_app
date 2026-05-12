<p align="center">
  <img src="frontend/public/icon-512.png" alt="Cube Practice logo" width="128" />
</p>

# Cube Practice

*A quiet place to drill.*

A spaced-repetition app for Rubik's cube algorithms. Pick a case, run the algorithm, grade yourself, and let the scheduler bring it back when you're about to forget it.

**Live at [cube.nebulouscode.com](https://cube.nebulouscode.com)** · **Status:** MVP feature-complete; final pre-launch pass in progress.

**Stack:** Rust + Axum + sqlx (backend), Vue 3 + Vite + TypeScript (frontend), Postgres on Neon, deployed on Render.

## Screenshots

<!--
Screenshots live in docs/screen_shots/. Drop PNGs at the paths below with
the suggested filenames and they'll render here. Recommended viewport for
captures: 1280×800 desktop or 390×844 mobile.
-->

![Landing page](docs/screen_shots/landing.png)
*Landing page — first impression for a cold visitor. Should show the hero, the value prop, and the call-to-action that lets someone try guest mode without signing up.*

![Practice dashboard](docs/screen_shots/practice_dashboard.png)
*Practice dashboard — the daily landing screen for a signed-in user. Capture this with a few cards already due so the standing card, due count, and study streak chip are all visible.*

![Study session](docs/screen_shots/study_session_revealed.png)
*A study session mid-reveal — capture the pattern diagram, the algorithm, and the 4-button (Fail / Hard / Good / Easy) grading row. This is the core feature of the app, so it's the most important shot.*

![Cases browser](docs/screen_shots/cases_browser.png)
*Cases browser with filter chips active — capture with a primary-shape filter and at least one state chip selected so the filter UI is doing visible work, not sitting empty.*

![Progress view](docs/screen_shots/progress.png)
*Progress view — the case-state breakdown (not-started / learning / due / mastered). Capture an account with enough practice history to show non-trivial distribution across all four states.*

## Repo layout

```
backend/         Rust + Axum API (cargo)
frontend/        Vue 3 + Vite + TypeScript SPA (npm)
docs/            ARCHITECTURE, CHANGELOG, milestones, policies, post-MVP backlog
initial_design/  React prototype — historical reference, no longer load-bearing
tools/           test.sh and other dev/CI helpers
```

## Prerequisites

- Rust (stable, 1.95+) — `rustup update stable`
- Node 24+ and npm 11+
- Postgres 16+ — a local instance or a Neon connection string

## Run the backend

```
cd backend
cp .env.example .env  # fill in DATABASE_URL, JWT_SECRET, RESEND_*, RECAPTCHA_*
cargo run             # debug profile, fast compile
# or
cargo run --release   # release profile — what Render runs in production
```

Migrations run automatically on startup via `sqlx::migrate!` — no separate migration step.

Health check: `curl http://localhost:8080/api/v1/health` → `{"status":"ok"}`.

Run the test suite:

```
cargo test                # all tests, no coverage
tools/test.sh             # backend coverage + frontend type-check + tests (mirrors CI)
tools/test.sh --enforce   # also enforce the 95% coverage gate (CI does this)
```

Tests need a separate `TEST_DATABASE_URL` so a misconfigured run can't accidentally drop production tables. See `backend/.env.example` for the full env-var list (DATABASE_URL, JWT_SECRET, ARGON2_*, RESEND_*, RECAPTCHA_*, TEST_DATABASE_URL).

### Render deployment

The backend Render service is configured as:

- **Build:** `cargo build --release`
- **Start:** `cargo run --release`

`cargo run --release` is preferred over invoking the binary by path so a future package rename doesn't require touching Render config.

## Run the frontend

```
cd frontend
npm install   # first time only
npm run dev
```

Visit `http://localhost:5173`. Lint, type-check, unit tests, and production build:

```
npm run lint
npm run type-check
npm run test:unit
npm run build
```

## Where to look first

- `docs/ARCHITECTURE.md` — live as-built reference: schema, auth design, API contract, frontend routes
- `docs/CHANGELOG.md` — what shipped when, milestone by milestone
- `docs/TODO.md` — current hard-blocker list before launch
- `docs/POST_MVP.md` — live backlog of work that's intentionally not in MVP
- `docs/concepts/` — evergreen "why" docs (SM-2-vs-Anki, OLL case reference)
- `docs/milestones/` — per-phase design docs (00 is the original spec; 01–07 are the MVP build)
- `docs/policies/` — Privacy Policy and Terms of Service

## License

Source-available under the **PolyForm Noncommercial License 1.0.0** — see [LICENSE](LICENSE) for the full text. In short: anyone may read, run, modify, and contribute to this code for any noncommercial purpose. Commercial use of the code itself (including standing up a competing hosted version for resale) is reserved to the maintainer. The hosted service at cube.nebulouscode.com is governed separately by the [Terms of Service](docs/policies/terms_of_service.md).

Forks and pull requests are welcome but not expected given the license. If you do open one, you grant the maintainer the right to relicense your contribution under any future license.
