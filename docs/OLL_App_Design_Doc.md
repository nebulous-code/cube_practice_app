# OLL Practice App — Design Document

## 1. Product Scope

### What It Is
A web-based spaced repetition flashcard app for Rubik's cube algorithm practice. Modeled on Anki's core study loop but purpose-built for OLL (and future solve stages). Users study cases, grade themselves with a 4-button rating (Fail / Hard / Good / Easy), and an Anki-style modified SM-2 algorithm schedules future reviews. See `docs/sm2_vs_anki_summary.md` for why the Anki variant rather than canonical SM-2.

### MVP Features
- Email/password registration with email verification and reCAPTCHA
- Full login, logout, and password reset flow
- Study mode: all due cards per the schedule
- Free study mode: browse and practice any case regardless of schedule
- Filter study by Tier 1 tag, Tier 2 tag, user-defined tags, or progress state
- 4-button grading per review (Fail / Hard / Good / Easy), Anki-style SM-2 calculates next interval
- OLL diagrams rendered dynamically from a 9-character pattern string per case
- Per-user overrides for algorithm, result mapping (case + rotation), nickname, and Tier 2 tag
- User-defined free-form tags per case
- Progress dashboard: due today, learning, mastered, not started
- Daily study streak tracking
- Mobile-friendly web app (mobile-shaped layout on desktop is acceptable for MVP)

### Post-MVP
- PLL, F2L, and other solve stage expansion
- Other cube types (4x4, Megaminx, etc.)
- Stats over time and progress graphs (a placeholder/skeleton view ships in MVP so users know it's coming)
- Admin panel
- Public case browser (no login required)
- Dark mode

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
result_rotation  INT NOT NULL DEFAULT 0       -- 0=none, 1=cw, 2=180, 3=ccw (quarter-turns CW)
diagram_data     JSONB NOT NULL    -- 9-character pattern string used by the dynamic renderer (see §9)
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
streak_count        INT NOT NULL DEFAULT 0   -- consecutive practice days; updated on each review
last_practice_date  DATE                     -- last day the user submitted a review (NULL = never)
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
result_rotation  INT              -- user's override rotation (0–3 quarter-turns CW), NULL = use default
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
last_grade      INT              -- 0=Again, 1=Hard, 2=Good, 3=Easy
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

## 4. Spaced Repetition Algorithm (Anki-style SM-2)

The data shape (`ease_factor`, `interval_days`, `repetitions`, `due_date`) matches canonical SM-2, but the update rule is Anki's modified version. See `docs/sm2_vs_anki_summary.md` for the rationale.

### Inputs
A user grade per review, one of four values:

| Code | Button | Meaning |
|-----:|--------|---------|
| 0 | Fail  | Failed — couldn't recall |
| 1 | Hard  | Recalled, but with difficulty |
| 2 | Good  | Recalled cleanly |
| 3 | Easy  | Recalled easily, felt too easy |

### Update rule
```
Input: ease_factor, interval_days, repetitions, grade ∈ {0,1,2,3}

If grade == 0 (Fail):
    repetitions = 0
    interval_days = 1
    ease_factor -= 0.20

Else:
    if repetitions == 0:
        interval_days = 1
    elif repetitions == 1:
        interval_days = 6
    else:
        if grade == 1 (Hard):
            interval_days = round(interval_days * HARD_INTERVAL_MULT)
        elif grade == 2 (Good):
            interval_days = round(interval_days * ease_factor)
        elif grade == 3 (Easy):
            interval_days = round(interval_days * ease_factor * EASY_BONUS)

    repetitions += 1

    Ease delta:
        Hard: ease_factor -= 0.15
        Good: ease_factor unchanged
        Easy: ease_factor += 0.15

ease_factor = max(EASE_FLOOR, ease_factor)
due_date = today + interval_days
```

### Constants
| Constant | Value | Notes |
|----------|------:|-------|
| Initial `ease_factor` | 2.5 | Default for a new card |
| `EASE_FLOOR` | 1.3 | Minimum ease |
| `HARD_INTERVAL_MULT` | 1.2 | Anki default |
| `EASY_BONUS` | 1.3 | Anki default |
| Fail ease delta | −0.20 | Anki default |

### Why this differs from canonical SM-2
Canonical SM-2 uses 0–5 grades with a smooth quadratic ease formula and treats `q < 3` as failure. The Anki variant collapses to 4 buttons, uses flat ease deltas (±0.15), and isolates the failure case to the single Fail button — friendlier UX and avoids "ease hell" (cards that fail repeatedly drop ease very low and then keep failing).

### Behavioral notes
- **Learning steps** (Anki's `1m / 10m` intermediate intervals before SM-2 kicks in) are not implemented. New cards go directly into the schedule on first review (rep 0 → 1d, rep 1 → 6d, then × ease).
- **Late-review bonus** (Anki's bonus for cards reviewed past their due date) is not implemented.
- **New cards** stay in `not_started` and are not auto-promoted into the due queue. The user explicitly starts a card from the case browser/detail; the first review creates the `user_case_progress` row.
- **Streak update**: on each review submission, if `last_practice_date` is yesterday → `streak_count += 1`; if today → no change; otherwise → `streak_count = 1`. Then set `last_practice_date = today`.

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
| POST | `/study/:case_id/review` | Required | Submit a grade (0–3 — Fail/Hard/Good/Easy, see §4), updates progress |

Query params for `/study/free`:
- `tier1_tag` — filter by +, -, L, *
- `tier2_tag` — filter by T_shapes, knight_move, etc.
- `tag` — filter by user-defined tag name
- `status` — `not_started`, `learning`, `due`, `mastered` (thresholds derived from `repetitions` / `interval_days` / `due_date` — see `docs/outstanding_decision.md` §1.3)

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

### Styling
- Vue Single-File Components (`.vue`) using `<style scoped>` blocks. No CSS-in-JS, no utility framework, no component library.
- Shared design tokens (palette, fonts, radii) live in a small `tokens.css` ported from `initial_design/src/ui.jsx` (`paper` and `fonts` constants).
- Light mode only for MVP.

### Pinia Stores
- `authStore` — current user, login/logout actions
- `casesStore` — all cases with user overrides merged
- `studyStore` — current study session state, grading queue
- `progressStore` — SM-2 data per case
- `tagsStore` — user-defined tags

---

## 9. Diagram Spec

### Dynamic Pattern Rendering (MVP)
Diagrams are rendered dynamically by a Vue `<PatternDiagram>` component from a 9-character pattern string stored on each case. Reference implementation is the React prototype's `initial_design/src/diagram.jsx` — port the same rendering logic to Vue.

The pattern string encodes the OLL face plus side strips. 9 characters, laid out as a 3×3 grid (top-left to bottom-right). Each char is one of:
- `X` — yellow on top
- `T` / `L` / `R` / `B` — sticker faces top / left / right / back side respectively
- (other) — non-yellow top sticker, no side flap

The string is stored in `cases.diagram_data` (JSONB) so the field can be extended later for stages with richer diagram data (e.g. PLL/F2L) without a schema change.

The result mapping (back of card) references another case by `result_case_id` plus a `result_rotation` integer (0–3 quarter-turns clockwise). The frontend applies CSS `transform: rotate(90deg * n)` to the rendered SVG — no additional images needed.

**Rendering spec:** mirror the prototype's existing visual treatment. The component is responsible for size, aspect, and colors. Restyling later (theming, dark mode, larger displays) is straightforward without re-exporting any image assets.

### Why dynamic rather than pre-built SVGs
Static SVGs were the original plan, but dynamic rendering is simpler operationally — no asset pipeline, no naming conventions, no manual export step when a case representation is tweaked. Performance is fine: 57 small SVGs rendered client-side from a 9-char input. Keeping the renderer also leaves room to restyle without re-exporting images.

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
