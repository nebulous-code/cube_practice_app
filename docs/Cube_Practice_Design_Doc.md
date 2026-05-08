# Cube Practice App — Design Document

## 1. Product Scope

### What It Is
A web-based spaced repetition flashcard app for Rubik's cube algorithm practice. Modeled on Anki's core study loop. The MVP ships OLL (Orientation of the Last Layer) on a 3×3, with the schema and API designed to expand to PLL, F2L, and other stages/puzzles post-MVP. Users study cases, grade themselves with a 4-button rating (Fail / Hard / Good / Easy), and an Anki-style modified SM-2 algorithm schedules future reviews. See `docs/sm2_vs_anki_summary.md` for why the Anki variant rather than canonical SM-2.

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
- Guest mode (last MVP feature shipped — see `docs/guest_mode_design_doc.md`)

### Post-MVP
- PLL, F2L, and other solve stage expansion
- Other cube types (4x4, Megaminx, etc.)
- Stats over time and progress graphs (a placeholder/skeleton view ships in MVP so users know it's coming)
- Admin panel
- Public case browser (no login required)
- Dark mode
- Additional public-facing marketing pages (features, pricing, FAQ, etc.) beyond the M5 landing page
- Full accessibility audit — screen-reader walkthrough, ARIA live regions, complete WCAG AA review (M5 ships a basic pass: keyboard nav, focus rings, form labels, spot-check contrast)
- **Per-user timezone + local-midnight rollover** for streak/due-date comparisons. MVP uses server UTC date for "today" — a user in PST sees streaks tick at 5 PM Pacific (00:00 UTC). Post-MVP: store `users.timezone`, roll over at user-local midnight. Two reviews near UTC midnight currently can fall on different "today" values; that goes away with per-user rollover.
- **"Download my data" / data export before account deletion.** Deferred from M7 (`docs/milestones/07_delete_account.md`). MVP delete is straight hard-delete; a JSON dump endpoint + Settings-side download flow can layer in once explicit user demand surfaces.
- **Guest mode "Discard guest data" Settings entry.** Deferred from M7. Trivially `clearGuestState()`, but the UX (confirmation pane, warning copy) deserves its own design pass — the existing M7 deletion flow is account-scoped and doesn't apply.
- **Free-study filters: disable chips with no remaining matches.** Today, picking "L" as the primary shape leaves every tag chip enabled even though tags like "knight_move" don't intersect with L cases — the user discovers this only by hitting "0 cases match". Post-MVP: gray out / hide chips whose addition would leave the result set empty given the current filter state. Same treatment for tags/state interactions.
- **Email reminders.** Opt-in daily/weekly nudge when the user has cards due. Needs a `users.reminder_preference` enum (off / daily / weekly), a per-user "last reminder sent" timestamp, and a worker pass that runs on a schedule. Resend integration already in place from M1 — wiring is the easy part; the design call is cadence, copy, and unsubscribe-link semantics.
- **Paid email subscription / premium tier.** Possible monetization path. Would need: subscription state on `users`, a billing provider (Stripe likely), gated features (TBD — possibly stats over time, additional puzzle types, or dark mode), and a customer-portal flow for plan changes / cancellations. Discovery work; not committed.
- **Easier-to-copy verification / reset codes in email.** Today the 6-digit code sits inside an HTML paragraph; on mobile the user has to long-press and trim whitespace. Wrap the code in a `<code>`/monospace box with generous padding so a single tap selects the whole code, or include a Markdown-style fenced block in the plaintext email body. Trivial template change once the design lands.

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
id                       UUID PRIMARY KEY
email                    TEXT UNIQUE NOT NULL
pending_email            TEXT             -- new email awaiting verification after a profile email change; NULL when none
display_name             TEXT NOT NULL    -- shown on avatar/settings; captured at registration
password_hash            TEXT NOT NULL
email_verified           BOOLEAN NOT NULL DEFAULT FALSE
verification_code        TEXT             -- 6-digit code emailed for verification; NULL when none active
verification_code_expires TIMESTAMPTZ     -- 10-minute TTL from issue
reset_code               TEXT             -- 6-digit code emailed for password reset; NULL when none active
reset_code_expires       TIMESTAMPTZ      -- 1-hour TTL from issue
streak_count             INT NOT NULL DEFAULT 0   -- consecutive practice days; updated on each review
last_practice_date       DATE                     -- last day the user submitted a review (NULL = never)
created_at               TIMESTAMPTZ
updated_at               TIMESTAMPTZ
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

All verification and reset flows use **6-digit numeric codes** entered into the app rather than links in emails. This is friendlier on mobile (no app-switching) and matches the auth mockups. reCAPTCHA is reCAPTCHA v3 (invisible — no UI element, just a token submitted with form posts).

### Registration Flow
1. User submits `display_name`, email, password, reCAPTCHA token
2. Backend verifies reCAPTCHA with Google's API
3. Password hashed with argon2id
4. User row inserted with `email_verified = false`, a generated 6-digit `verification_code`, and `verification_code_expires = now + 10 minutes`
5. Verification email sent containing the 6-digit code
6. User enters the code on the verify screen → backend validates code + expiry, sets `email_verified = true`, clears the code fields
7. **On success the verify endpoint sets the JWT cookie** — the user is logged in directly and lands in onboarding/dashboard. No separate sign-in step.

### Login Flow
1. User submits email and password
2. Backend fetches user by email, verifies password with argon2
3. Checks `email_verified = true` — if not, returns 403 with `{ reason: "email_not_verified" }` and the frontend routes to the verify screen with a fresh `resend-verification` call
4. Generates JWT: `{ sub: user_id, exp: now + 30 days, iat: now }`
5. Signs with HS256 using `JWT_SECRET` environment variable
6. Sets JWT as httpOnly, Secure, SameSite=Strict cookie
7. Returns 200 with basic user info (id, email, display_name) — no JWT in response body

### Per-Request Auth
1. Axum `AuthUser` extractor reads the httpOnly cookie
2. Decodes and validates JWT signature with `JWT_SECRET`
3. Checks expiry
4. Looks up the matching `sessions` row by token hash; rejects if `revoked = true`
5. Extracts `user_id` from claims
6. Injects into handler or returns 401

### Logout
- Backend clears the httpOnly cookie
- Marks the current session row as revoked in the `sessions` table

### Sign Out Everywhere
- `POST /auth/sign-out-all` with body `{ current_password }`
- Backend verifies `current_password` with argon2 before doing anything (defense against an attacker with temporary access to a logged-in device)
- On success: sets `revoked = true` on every session row for the current user (including the current one), clears the JWT cookie, returns 200
- On wrong password: returns 401, no sessions touched
- Frontend redirects to `/login` on success

### Resend Verification Code
- `POST /auth/resend-verification` — generates a new `verification_code`, replaces the stored one, resets `verification_code_expires`, and emails the new code
- Rate-limited to 1 request per minute per user

### Password Reset Flow
1. User submits email on the forgot-password screen
2. Backend generates a 6-digit `reset_code` with `reset_code_expires = now + 1 hour`, stores on user row. Endpoint is idempotent — calling it again overwrites the existing code
3. Reset email sent containing the 6-digit code
4. User enters code + new password on the reset screen
5. Backend validates code + expiry, hashes new password, clears `reset_code` and `reset_code_expires`
6. User logs in with the new password (no auto-login on reset — they explicitly enter the new password on the login screen)

### Change Password (logged-in user)
- `POST /auth/change-password` with `{ current_password, new_password }`
- Backend verifies `current_password` with argon2
- Hashes and stores `new_password`
- JWT cookie is unchanged — user stays logged in on this device. Other devices stay logged in too unless the user follows up with `sign-out-all`

### Profile Update (display name, email)
- `PATCH /auth/me` accepts `{ display_name?, email? }`
- `display_name` updates write-through immediately
- **Email change uses a `pending_email` flow:**
  1. New email is written to `users.pending_email` (NOT `email`)
  2. A new `verification_code` is generated for the pending email and emailed there
  3. The user keeps their existing `email`, `email_verified` stays `true`, and the user remains logged in
  4. Frontend shows a banner: "Verify your new email <pending_email> to switch addresses." with a resend button
  5. When the user submits the verification code, the verify endpoint detects the `pending_email`, copies it into `email`, clears `pending_email` and the code fields, and confirms success
  6. Until verified, login still works with the original email; no risk of typo lockout

### Token Lifetime
- 30-day JWT, no refresh token
- Activity does not reset the clock (simplest implementation)
- If security becomes a concern post-MVP, sliding expiry or refresh tokens can be added

### Frontend Route Guard
- Any unauthenticated request to a protected route redirects to `/login?next=<original-path>`
- After successful login, the frontend reads `next` and routes there (default `/`)
- Any authenticated request to `/login`, `/register`, `/forgot-password`, `/reset-password` redirects to `/`
- `/verify-email` is reachable both authenticated (during email change) and unauthenticated (during initial registration if the user closes the tab and comes back)

### Splash Screen
- The frontend renders a logo splash on initial app load while the cookie is exchanged for `/auth/me`
- Minimum display time: 800ms (so the splash doesn't flicker on a fast response)
- Maximum: until `/auth/me` resolves (success → app, 401 → `/login`)

### Static Pages
- `/about`, `/terms`, `/privacy`, `/acknowledgements` are static frontend pages — no API
- Reachable from the login footer, registration footer ("By creating an account you agree to our Terms…"), and the Settings → About section
- Terms and Privacy content is required pre-launch (tracked in `docs/TODO.md`)
- Acknowledgements can auto-generate from `package.json` / `Cargo.toml` license metadata; manual content is fine for MVP

---

## 6. API Contract

All endpoints are prefixed `/api/v1`. All request/response bodies are JSON. Protected routes require the httpOnly JWT cookie.

### Auth

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| POST | `/auth/register` | None | Register new user (display_name, email, password, reCAPTCHA token) |
| POST | `/auth/verify-email` | Optional | Verify email with 6-digit code; sets JWT cookie on initial verification; for email-change verification, requires existing auth and promotes `pending_email` |
| POST | `/auth/resend-verification` | Optional | Resend the verification code; rate-limited 1/min per user. Auth optional — accepts either the current session or `{ email }` body for the unauthenticated initial-verification case |
| POST | `/auth/login` | None | Login, sets cookie. Returns 403 `{ reason: "email_not_verified" }` if email is not yet verified |
| POST | `/auth/logout` | Required | Clears cookie, revokes the current session row |
| POST | `/auth/sign-out-all` | Required | Requires `{ current_password }` body. On success revokes all session rows for the user (including the current one) and clears the cookie |
| POST | `/auth/forgot-password` | None | Send 6-digit reset code by email; idempotent |
| POST | `/auth/reset-password` | None | Submit new password with 6-digit code |
| POST | `/auth/change-password` | Required | Change password while logged in (current_password, new_password) |
| GET | `/auth/me` | Required | Get current user identity: `{ id, email, display_name, pending_email, email_verified }`. Stats live on `/progress` to keep auth/PII separated from app metrics |
| PATCH | `/auth/me` | Required | Update profile (display_name?, email?). Email change writes to `pending_email` and emails a verification code |

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
| GET | `/progress` | Required | Summary: `{ due_today, learning, mastered, not_started, streak_count, last_practice_date }`. Stats endpoint — kept separate from `/auth/me` so identity/PII stays disjoint from app metrics. New stats fields land here as the app grows |
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
| `/register` | RegisterView | No | Registration form with invisible reCAPTCHA v3 |
| `/verify-email` | VerifyEmailView | Optional | 6-digit code entry; same view used for initial verification and email-change verification |
| `/forgot-password` | ForgotPasswordView | No | Request 6-digit password reset code |
| `/reset-password` | ResetPasswordView | No | Submit code + new password |
| `/` | DashboardView | Yes | Progress summary, due cards count, quick start study |
| `/study` | StudyView | Yes | SM-2 study mode, due cards |
| `/study/free` | FreeStudyView | Yes | Free study with filters |
| `/cases` | CaseBrowserView | Yes | Browse all 57 cases |
| `/cases/:id` | CaseDetailView | Yes | Single case detail, edit overrides |
| `/progress` | ProgressView | Yes | Full progress breakdown per case |
| `/settings` | SettingsView | Yes | Account settings, password change, sign out, sign out everywhere |
| `/about` | AboutView | No | Static — version, links to terms/privacy/acknowledgements |
| `/terms` | TermsView | No | Static — Terms of Service content |
| `/privacy` | PrivacyView | No | Static — Privacy Policy content |
| `/acknowledgements` | AcknowledgementsView | No | Static — third-party license credits |

Splash is rendered before the router resolves, while `/auth/me` is in flight on initial load. See §5 "Splash Screen" for timing.

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
4. Auth endpoints — register, verify-email (with code), resend-verification, login, logout, sign-out-all, forgot-password, reset-password, change-password, GET/PATCH `/auth/me`
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
21. Static pages — `/about`, `/terms`, `/privacy`, `/acknowledgements` (placeholder content fine until launch)
22. Guest mode — final MVP feature, per `docs/guest_mode_design_doc.md`
23. Deployment — Render setup, environment variables, CI/CD
