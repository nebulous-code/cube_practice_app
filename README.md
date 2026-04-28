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
cargo run
```

Health check: `curl http://localhost:8080/api/v1/health` → `{"status":"ok"}`.

Run the test suite (once tests exist):

```
cargo test
```

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

- `docs/OLL_App_Design_Doc.md` — full product/spec
- `docs/milestones/README.md` — milestone breakdown
- `docs/milestones/01_auth_and_accounts.md` — current milestone's design + story list
- `docs/TODO.md` — items on the user's plate (terms/privacy content, account provisioning, etc.)
