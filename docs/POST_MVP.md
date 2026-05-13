# Post-MVP backlog

Live list of work that is intentionally *not* in MVP. Originally lived inside `milestones/00_initial_design_doc.md` §1, but the design doc is a frozen historical artifact and this list keeps growing — so it lives here now.

Each entry is "what + why-it's-deferred + rough shape if obvious." Nothing here is committed; this is the queue we pull from when deciding what to build next, not a roadmap with dates.

## Scope expansion

- PLL, F2L, and other solve-stage expansion.
- Other cube types (4×4, Megaminx, etc.).
- Mobile app (PWA or Capacitor, no native apps)
- Other utils (timer, solver, etc.)
- Improved Tutorial

## Product surface

- Stats over time and progress graphs (a placeholder/skeleton view ships in MVP so users know it's coming).
- Dark mode.
- FAQ page (very low priority).
- Full accessibility audit — screen-reader walkthrough, ARIA live regions, complete WCAG AA review. M5 ships a basic pass: keyboard nav, focus rings, form labels, spot-check contrast.
- **Avatar.** Allow users to pick an OLL case as their avatar in the style of the logo.

## Study mechanics

- **Per-user timezone + local-midnight rollover** for streak/due-date comparisons. MVP uses server UTC date for "today" — a user in PST sees streaks tick at 5 PM Pacific (00:00 UTC). Post-MVP: store `users.timezone`, roll over at user-local midnight. Two reviews near UTC midnight currently can fall on different "today" values; that goes away with per-user rollover.
- **Free-study filters: disable chips with no remaining matches.** Today, picking "L" as the primary shape leaves every tag chip enabled even though tags like "knight_move" don't intersect with L cases — the user discovers this only by hitting "0 cases match". Post-MVP: gray out / hide chips whose addition would leave the result set empty given the current filter state. Same treatment for tags/state interactions.

## Account / data

- **"Download my data" / data export before account deletion.** Deferred from M7 (`milestones/07_delete_account.md`). MVP delete is straight hard-delete; a JSON dump endpoint + Settings-side download flow can layer in once explicit user demand surfaces.
- **Guest mode "Discard guest data" Settings entry.** Deferred from M7. Trivially `clearGuestState()`, but the UX (confirmation pane, warning copy) deserves its own design pass — the existing M7 deletion flow is account-scoped and doesn't apply.

## Email / notifications

- **Email reminders.** Opt-in daily/weekly nudge when the user has cards due. Needs a `users.reminder_preference` enum (off / daily / weekly), a per-user "last reminder sent" timestamp, and a worker pass that runs on a schedule. Resend integration already in place from M1 — wiring is the easy part; the design call is cadence, copy, and unsubscribe-link semantics.
- **Easier-to-copy verification / reset codes in email.** Today the 6-digit code sits inside an HTML paragraph; on mobile the user has to long-press and trim whitespace. Wrap the code in a `<code>`/monospace box with generous padding so a single tap selects the whole code, or include a Markdown-style fenced block in the plaintext email body. Trivial template change once the design lands.

## Infrastructure / ops

- **Cold-start UX safety net.** Render free tier spins down after ~15 min idle; the next request stalls 20–60s while the dyno wakes. The splash screen handles the boot case, but in-session API calls (grade, reveal, etc.) currently look frozen. Two layered ideas:
  1. **Top progress bar (NProgress-style).** ~2px bar at the top of the viewport, animates while any axios request is in flight. Axios interceptors increment/decrement a counter; bar component watches it. Calibrated so it's invisible on a 200ms request and visibly creeps on a 30s one. Zero visual cost on fast requests, immediate signal on slow ones.
  2. **Subtle full-screen overlay after ~8 seconds.** When a request has been in flight that long, fade in a 30–50% black overlay with a small note: "still working — the server takes up to a minute to wake up after a quiet period." Dismisses on response. Threshold tuned to fire effectively only on cold-start, not on slow mobile networks.

  Either of these makes the spin-down tolerable. The "real fix" is a paid Render tier (~$5/mo) once the project has any monetization or donation path.

- **Operator stats — weekly user count + recent activity.** Mostly a curiosity feature: "is anyone using this, or is it just me?" Cheapest path is a `tools/operator_stats.sh` script that hits prod Neon read-only and prints user count + most recent sign-ins. Step up from there is a GitHub Action on a weekly schedule that runs the same queries and posts the result somewhere private (Discord webhook, private Gist, or just emails me@nebulouscode.com). Not worth a full admin UI for what's a couple of `SELECT count(*)` queries.

## Privacy / compliance

A grab-bag of ways to tighten the app's privacy posture and make the forthcoming Privacy Policy easier to defend. Not all of these will get done — they're noted so the decision to skip any one of them is deliberate rather than accidental.

- **Add error tracking thoughtfully (self-hosted GlitchTip or Sentry) or commit publicly to not adding it.** Right now we have neither, which means real production errors disappear silently. Path A: self-host GlitchTip on a $5 box so error payloads never leave our infra; we'd need to scrub user emails / IDs from breadcrumbs before ingest. Path B: write "no error tracking is wired up" into the privacy policy and live with the operational cost. Either is defensible; the current "vaguely meaning to add Sentry" middle state is not.
- **Reduce session cookie lifetime / add idle expiry.** Current JWT is a 30-day absolute lifetime with no server-side idle timeout. Options: drop absolute lifetime to 7 days, add a 14-day idle timeout enforced on the server (`sessions.last_seen_at`, refreshed lazily on each authenticated request), or rolling expiry that re-issues the JWT on active use. Trade-off is "log me back in" friction vs. the size of the steal-cookie blast radius.
- **`SECURITY.md` + responsible-disclosure email.** Standard hygiene for any public-facing app. A `SECURITY.md` at the repo root pointing at a `security@nebulouscode.com` (or similar) tells researchers where to send a quiet heads-up rather than opening a public GitHub issue. Coupled with a one-line acknowledgement section that names them once the patch ships, this is one of the cheapest professionalism wins.
- **`/.well-known/privacy.txt` and/or Global Privacy Control signal.** Two adjacent ideas: ship a `privacy.txt` at a well-known path pointing at the policy, contact, and jurisdiction, *and/or* honor the `Sec-GPC: 1` browser header by treating it as a do-not-sell / do-not-share signal at the API layer. We don't sell data today so GPC is mostly a posture statement, but it's also future-proofing against ever adding analytics.
- **Switch transactional email off Resend.** Resend is fine but it's another US-based subprocessor that sees every user's email address + the contents of verification/reset emails. Options: a more privacy-aligned managed provider like Postmark or Tutanota's transactional product (drop-in swap), or a self-hosted MTA (Postal, Maddy, or even raw Postfix on a $5 box). The self-host path is meaningfully more work (DNS, DKIM, SPF, DMARC, deliverability monitoring) and probably only justified if the privacy posture matters enough to absorb the operational cost.
- **Stop using JWTs for sessions; use opaque session tokens.** JWTs are fine in our shape (we already track a `sessions` row with a token hash so sign-out-all works), but the JWT itself is structurally a bearer credential that's hard to revoke server-side without the parallel sessions table. Switching to opaque random tokens (just the `sessions.token_hash` UUID, validated by DB lookup on each request) collapses the design — one source of truth for "is this session alive", no JWT-secret-rotation problem, no `exp`/`iat` claim drift. Cost is one DB hit per authenticated request (already happening for user lookup anyway).
## Testing

- **HTTP integration tests for route handlers.** Today we unit-test the lib functions each handler delegates to (high coverage on the business logic) but not the HTTP plumbing — auth gating, request validation, JSON response shape, status codes, cookie attachment. The handler-layer files (`routes/*`, `auth/extractor.rs`, `auth/session.rs`, `error.rs`'s `IntoResponse`) are excluded from the 95% coverage gate for that reason. Adding tower-based in-process integration tests (spin up the Axum app, send real `Request`s via `tower::ServiceExt::oneshot`, assert responses) would close the gap. Cost: ~30–50% more test code, slower CI; benefit: catches handler-level regressions (renamed JSON fields, dropped auth gates, wrong status codes) that lib-level tests can't see. Worth doing once the project has more than one developer, or on the first regression that the lib tests miss.

---

## Out of scope (MVP)

These were called out at design time as explicitly *not* the direction of the product, not just "not yet":

- Native mobile app.
- Social features.
- Sharing custom decks between users.
- Monetization / paid tier.
- Multi-region data residency (EU branches, region-pinned routing, etc.).
