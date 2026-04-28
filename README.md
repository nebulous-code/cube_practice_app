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

- `docs/Cube_Practice_Design_Doc.md` — full product/spec
- `docs/milestones/README.md` — milestone breakdown
- `docs/milestones/01_auth_and_accounts.md` — current milestone's design + story list
- `docs/TODO.md` — items on the user's plate (terms/privacy content, account provisioning, etc.)
