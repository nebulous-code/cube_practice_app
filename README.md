# Cube Practice App

An app to practice Rubik's cube algorithms and other puzzle cube algorithms.

## Repo layout

```
backend/         Rust + Axum API (cargo)
frontend/        Vue 3 + Vite + TypeScript SPA (npm)
docs/            Design docs, milestones, decisions logs, TODO
initial_design/  React prototype — reference for the Vue port
```

## Prerequisites

- Rust (stable, 1.95+) — `rustup update stable`
- Node 24+ and npm 11+
- A local Postgres or a Neon connection string for the backend (optional until milestone-1 endpoint work begins)

## Run the backend

```
cd backend
cp .env.example .env  # fill in DATABASE_URL etc as needed
cargo run             # debug profile, fast compile
# or
cargo run --release   # release profile — what Render runs in production
```

Health check: `curl http://localhost:8080/api/v1/health` → `{"status":"ok"}`.

Run the test suite (once tests exist):

```
cargo test
```

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

Visit `http://localhost:5173`. Lints + type-check + build:

```
npm run lint
npm run type-check
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
