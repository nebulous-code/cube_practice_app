# OLL Practice App — Design Document

## 1. Product Scope

### What It Is
A web-based spaced repetition flashcard app for Rubik's cube algorithm practice. Modeled on Anki's core study loop but purpose-built for OLL (and future solve stages). Users study cases, grade themselves 0–5, and the app uses the SM-2 algorithm to schedule future reviews.

### MVP Features
- Email/password registration with email verification and reCAPTCHA
- Full login, logout, and password reset flow
- Study mode: due cards per SM-2 schedule
- Free study mode: browse and practice any case regardless of schedule
- Filter study by Tier 1 tag, Tier 2 tag, user-defined tags, or progress state
- 0–5 grading per review, SM-2 calculates next interval
- OLL diagrams rendered as SVG (proven spec, 57 images rotated as needed)
- Per-user overrides for algorithm, result mapping (case + rotation), and nickname
- User-defined free-form tags per case
- Progress dashboard: due today, learning, mastered, not started
- Mobile-friendly web app (no App Store deployment)

### Post-MVP
- PLL, F2L, and other solve stage expansion
- Other cube types (4x4, Megaminx, etc.)
- Stats over time and progress graphs
- Admin panel
- Study streak tracking
- Public case browser (no login required)

### Out of Scope (MVP)
- Native mobile app
- Social features
- Sharing custom decks between users

---

## 2. Expansion-Proof Architecture Principle

The schema and API are designed around the general concept of **puzzle → stage → case**, not around OLL specifically. Adding PLL or F2L means inserting rows into existing tables, not creating new ones. The frontend rendering logic is the only thing that needs to evolve for stages with different diagram types (e.g. F2L shows different faces).

---

## 3. Database Schema

### `puzzle_types`
```
id          UUID PRIMARY KEY
name        TEXT NOT NULL        -- e.g. "3x3", "4x4", "Megaminx"
created_at  TIMESTAMPTZ
```

### `solve_stages`
```
id               UUID PRIMARY KEY
puzzle_type_id   UUID REFERENCES puzzle_types(id)
name             TEXT NOT NULL    -- e.g. "OLL", "PLL", "F2L"
description      TEXT
display_order    INT
created_at       TIMESTAMPTZ
```

### `cases`
Global, canonical case data. These are the defaults all users inherit.
```
id               UUID PRIMARY KEY
solve_stage_id   UUID REFERENCES solve_stages(id)
case_number      INT NOT NULL     -- e.g. 1–57 for OLL
nickname         TEXT             -- default nickname (e.g. "Sune")
algorithm        TEXT NOT NULL    -- default algorithm string
result_case_id   UUID REFERENCES cases(id)   -- which case appears on the back
result_rotation  TEXT             -- NULL, "cw", "180", "ccw"
diagram_data     JSONB            -- the LXR/TXB grid notation
tier1_tag        TEXT NOT NULL    -- "+", "-", "L", "*" — fixed, geometric
tier2_tag        TEXT             -- "T_shapes", "knight_move", etc. — global default
created_at       TIMESTAMPTZ

UNIQUE(solve_stage_id, case_number)
```

### `users`
```
id                  UUID PRIMARY KEY
email               TEXT UNIQUE NOT NULL
password_hash       TEXT NOT NULL
email_verified      BOOLEAN DEFAULT FALSE
verification_token  TEXT
reset_token         TEXT
reset_token_expires TIMESTAMPTZ
created_at          TIMESTAMPTZ
updated_at          TIMESTAMPTZ
```

### `user_case_settings`
Per-user overrides. NULL in any column means fall back to the global default in `cases`.
```
id               UUID PRIMARY KEY
user_id          UUID REFERENCES users(id)
case_id          UUID REFERENCES cases(id)
nickname         TEXT             -- user's override nickname, NULL = use default
algorithm        TEXT             -- user's override algorithm, NULL = use default
result_case_id   UUID REFERENCES cases(id)   -- user's override result, NULL = use default
result_rotation  TEXT             -- user's override rotation, NULL = use default
tier2_tag        TEXT             -- user's override tier2 tag, NULL = use default
created_at       TIMESTAMPTZ
updated_at       TIMESTAMPTZ

UNIQUE(user_id, case_id)
```

### `user_case_progress`
SM-2 data per user per case.
```
id              UUID PRIMARY KEY
user_id         UUID REFERENCES users(id)
case_id         UUID REFERENCES cases(id)
ease_factor     FLOAT DEFAULT 2.5
interval_days   INT DEFAULT 1
repetitions     INT DEFAULT 0    -- consecutive correct reviews
due_date        DATE DEFAULT NOW()
last_grade      INT              -- 0–5, last grade given
last_reviewed   TIMESTAMPTZ
created_at      TIMESTAMPTZ
updated_at      TIMESTAMPTZ

UNIQUE(user_id, case_id)
```

### `tags`
User-defined free-form tags.
```
id          UUID PRIMARY KEY
user_id     UUID REFERENCES users(id)
name        TEXT NOT NULL
created_at  TIMESTAMPTZ

UNIQUE(user_id, name)
```

### `case_tags`
Junction table: which user-defined tags are applied to which cases.
```
id          UUID PRIMARY KEY
user_id     UUID REFERENCES users(id)
case_id     UUID REFERENCES cases(id)
tag_id      UUID REFERENCES tags(id)
created_at  TIMESTAMPTZ

UNIQUE(user_id, case_id, tag_id)
```

### `sessions`
For JWT revocation if needed (e.g. logout, account compromise).
```
id          UUID PRIMARY KEY
user_id     UUID REFERENCES users(id)
token_hash  TEXT NOT NULL        -- hash of the JWT, not the JWT itself
expires_at  TIMESTAMPTZ
revoked     BOOLEAN DEFAULT FALSE
created_at  TIMESTAMPTZ
```

---

## 4. SM-2 Algorithm

SM-2 is public domain. Implementation per review:

```
Input: current ease_factor, interval_days, repetitions, grade (0–5)

If grade < 3:
  repetitions = 0
  interval_days = 1

Else:
  if repetitions == 0: interval_days = 1
  if repetitions == 1: interval_days = 6
  if repetitions > 1:  interval_days = round(interval_days * ease_factor)
  repetitions += 1

ease_factor = ease_factor + (0.1 - (5 - grade) * (0.08 + (5 - grade) * 0.02))
ease_factor = max(1.3, ease_factor)   -- floor of 1.3

due_date = today + interval_days
```

The ease factor is per-user per-case and drifts over time based on grading history. Cards graded consistently high get longer intervals. Cards graded low get reset and reviewed sooner.

---

## 5. Auth Design

### Registration Flow
1. User submits email, password, reCAPTCHA token
2. Backend verifies reCAPTCHA with Google's API
3. Password hashed with argon2id
4. User row inserted with `email_verified = false` and a generated `verification_token`
5. Verification email sent with a link containing the token
6. User clicks link → backend sets `email_verified = true`, clears token
7. User can now log in

### Login Flow
1. User submits email and password
2. Backend fetches user by email, verifies password with argon2
3. Checks `email_verified = true`
4. Generates JWT: `{ sub: user_id, exp: now + 30 days, iat: now }`
5. Signs with HS256 using `JWT_SECRET` environment variable
6. Sets JWT as httpOnly, Secure, SameSite=Strict cookie
7. Returns 200 with basic user info (id, email) — no JWT in response body

### Per-Request Auth
1. Axum `AuthUser` extractor reads the httpOnly cookie
2. Decodes and validates JWT signature with `JWT_SECRET`
3. Checks expiry
4. Extracts `user_id` from claims
5. Injects into handler or returns 401

### Logout
- Backend clears the httpOnly cookie
- Optionally marks the session as revoked in the `sessions` table

### Password Reset Flow
1. User submits email
2. Backend generates a `reset_token` with a 1-hour expiry, stores on user row
3. Reset email sent with link containing token
4. User submits new password + token
5. Backend validates token and expiry, hashes new password, clears token
6. User logs in with new password

### Token Lifetime
- 30-day JWT, no refresh token
- Activity does not reset the clock (simplest implementation)
- If security becomes a concern post-MVP, sliding expiry or refresh tokens can be added

---

## 6. API Contract

All endpoints are prefixed `/api/v1`. All request/response bodies are JSON. Protected routes require the httpOnly JWT cookie.

### Auth

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| POST | `/auth/register` | None | Register new user |
| POST | `/auth/verify-email` | None | Verify email with token |
| POST | `/auth/login` | None | Login, sets cookie |
| POST | `/auth/logout` | Required | Clears cookie |
| POST | `/auth/forgot-password` | None | Send reset email |
| POST | `/auth/reset-password` | None | Submit new password with token |
| GET | `/auth/me` | Required | Get current user info |

### Cases

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/cases` | Required | Get all cases for current user (merged with their overrides) |
| GET | `/cases/:id` | Required | Get single case with user overrides applied |
| PATCH | `/cases/:id/settings` | Required | Update user's override for nickname, algorithm, result |

### Study

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/study/due` | Required | Get cases due today per SM-2 |
| GET | `/study/free` | Required | Get all cases for free study (filterable) |
| POST | `/study/:case_id/review` | Required | Submit a grade (0–5), updates SM-2 progress |

Query params for `/study/free`:
- `tier1_tag` — filter by +, -, L, *
- `tier2_tag` — filter by T_shapes, knight_move, etc.
- `tag` — filter by user-defined tag name
- `status` — `not_started`, `learning`, `due`, `mastered`

### Progress

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/progress` | Required | Summary: due today, learning, mastered, not started counts |
| GET | `/progress/cases` | Required | Full progress data for all cases |

### Tags

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/tags` | Required | Get all user-defined tags |
| POST | `/tags` | Required | Create a new tag |
| DELETE | `/tags/:id` | Required | Delete a tag |
| POST | `/cases/:id/tags` | Required | Apply a tag to a case |
| DELETE | `/cases/:id/tags/:tag_id` | Required | Remove a tag from a case |

---

## 7. API Access Control

The API should only accept requests from trusted client origins. This is a placeholder section — full design to be completed before post-MVP work, but the following approaches should be evaluated:

**CORS restriction** — already required. The Axum CORS layer will whitelist only the deployed Vue frontend origin and localhost in development. This prevents browser-based requests from unknown origins.

**API key for non-browser clients** — if a mobile app or third-party client is added post-MVP, a static API key scheme (sent as a header) should gate access. Keys would be stored in the `api_keys` table with per-key permissions and revocation support.

**Rate limiting** — `tower-http` or a custom Tower middleware layer should enforce per-IP and per-user rate limits on auth endpoints specifically (register, login, reset) to prevent brute force. General endpoints should also be rate limited post-MVP.

**Future consideration** — if the app becomes public-facing enough to warrant it, moving to OAuth2 client credentials for machine-to-machine auth is the production-grade solution.

---

## 8. Frontend Routes

Built with Vue 3 + Vite + Vue Router + Pinia.

| Route | Component | Auth Required | Description |
|-------|-----------|---------------|-------------|
| `/login` | LoginView | No | Email/password login form |
| `/register` | RegisterView | No | Registration form with reCAPTCHA |
| `/verify-email` | VerifyEmailView | No | Handles email verification link |
| `/forgot-password` | ForgotPasswordView | No | Request password reset |
| `/reset-password` | ResetPasswordView | No | Submit new password |
| `/` | DashboardView | Yes | Progress summary, due cards count, quick start study |
| `/study` | StudyView | Yes | SM-2 study mode, due cards |
| `/study/free` | FreeStudyView | Yes | Free study with filters |
| `/cases` | CaseBrowserView | Yes | Browse all 57 cases |
| `/cases/:id` | CaseDetailView | Yes | Single case detail, edit overrides |
| `/progress` | ProgressView | Yes | Full progress breakdown per case |
| `/settings` | SettingsView | Yes | Account settings, password change |

### Pinia Stores
- `authStore` — current user, login/logout actions
- `casesStore` — all cases with user overrides merged
- `studyStore` — current study session state, grading queue
- `progressStore` — SM-2 data per case
- `tagsStore` — user-defined tags

---

## 9. Diagram Spec

### Static SVG Files (MVP)
57 SVG files are pre-built, one per OLL case. Named `oll_{case_number:02}.svg`. These are static assets bundled with the Vue frontend — they do not change and do not need to be served from the API.

OLL cases are universal and fixed. The SVG files are already generated and proven. Dynamic diagram rendering is explicitly out of scope for MVP.

The result mapping (back of card) references another case's SVG by case number plus an optional rotation (`cw`, `180`, `ccw`, or null). The frontend applies CSS `transform: rotate()` to the referenced SVG — no additional image files needed. Only 57 files serve all 114 front/back combinations.

**SVG spec:**
- Canvas: ~142×142px, white background, `rx=8`
- Top face: 3×3 grid of 34×34px cells, `rx=1`
  - Yellow `#FFD700` for `X` cells
  - Light gray `#EEEEEE` for non-`X` cells
  - Medium gray `#DDDDDD` face background
- Side strips: 10px thick, yellow only where sticker faces that side, 2px gap from face, 8px outer padding
- Stroke: `#333333`, 0.8px

### Post-MVP Note
The `diagram_data` JSONB field in the `cases` table retains the raw LXR/TXB grid notation. This is not used by the MVP frontend but preserves the option to render diagrams dynamically in the future — for example when adding PLL or F2L cases that may have different diagram styles not covered by the existing SVG set.

---

## 10. Deployment Plan

### Email Provider: Resend
Resend is the chosen SMTP provider. It offers a permanent free tier of 3,000 emails/month (capped at 100/day) — sufficient for this app indefinitely at current scale. Paid tiers match SendGrid pricing if volume ever requires it.

**Important:** Resend requires domain verification which can take 1–2 business days. Set up and verify the sending domain before beginning auth implementation.

Resend exposes a simple REST API. From Rust, emails are sent via `reqwest` HTTP POST — no official Rust SDK is needed.

### Services (both on Render)
- **Frontend**: Static site, Vue build output
- **Backend**: Web service, Rust binary

### Environment Variables

**Backend:**
```
DATABASE_URL          -- Neon PostgreSQL connection string
JWT_SECRET            -- random 256-bit secret, never committed to repo
RECAPTCHA_SECRET_KEY  -- Google reCAPTCHA secret
RESEND_API_KEY        -- Resend API key
EMAIL_FROM            -- verified sender address (must match Resend domain)
FRONTEND_URL          -- for CORS whitelist and email link generation
RUST_LOG              -- log level, e.g. "info"
```

**Frontend:**
```
VITE_API_BASE_URL         -- Render backend URL
VITE_RECAPTCHA_SITE_KEY   -- Google reCAPTCHA public key
```

### Build Order
1. Neon DB provisioned, migrations run via sqlx-cli
2. Rust backend deployed to Render, health check confirmed
3. Vue frontend deployed to Render static site, pointed at backend URL
4. DNS / custom domain configured (post-MVP)

### CI/CD
GitHub Actions for both services:
- Backend: `cargo test` + `cargo build --release` on push to main
- Frontend: `vite build` on push to main
- Render autodeploy triggered on successful build

---

## 11. Build Order (MVP)

1. Rust project scaffold — Axum, sqlx, tower-http wired up, health check endpoint live
2. Database migrations — all tables created via sqlx migrate
3. Seed script — insert puzzle_types, solve_stages, and all 57 OLL cases with default data
4. Auth endpoints — register, verify email, login, logout, forgot/reset password
5. JWT middleware extractor — AuthUser, httpOnly cookie handling
6. Cases endpoints — GET all, GET one, PATCH settings (with override merge logic)
7. Study endpoints — due cards, free study with filters, review/grading with SM-2
8. Progress endpoints — summary and per-case breakdown
9. Tags endpoints — CRUD tags, apply/remove from cases
10. Vue project scaffold — Vite, Vue Router, Pinia, Axios
11. Auth views — login, register, verify email, forgot/reset password
12. Case data layer — fetch and store cases in Pinia with overrides merged
13. Study view — SM-2 card loop, grading UI, SVG diagrams with rotation
14. Free study view — same card UI, filter controls
15. Dashboard — due count, progress summary, quick-start button
16. Case browser — all 57 cases, filter by tags
17. Case detail — view overrides, edit nickname/algorithm/result mapping
18. Progress view — per-case SM-2 data
19. Settings — password change, account info
20. Tags UI — create/delete tags, apply to cases
21. Deployment — Render setup, environment variables, CI/CD
