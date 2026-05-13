# Launch Checklist — `quiet-cube.com`

Linear, top-to-bottom. Don't skip ahead — several steps assume earlier ones already landed. Check each box as you go.

**Target architecture:**

- Frontend at `https://quiet-cube.com` (Render static site)
- Backend at `https://api.quiet-cube.com` (Render web service)
- Both served from `main` branch on Render
- `cube.nebulouscode.com` → 301 → `quiet-cube.com` after cutover
- Dev environment moves to bare `*.onrender.com` URLs (no custom domain)

**Why `api.quiet-cube.com` and not the bare onrender.com URL on the backend:** the session cookie is `SameSite=Strict`. Frontend and backend must be on the same eTLD+1 for the browser to send the cookie. `onrender.com` is on the Public Suffix List, so every `*.onrender.com` is its own site — that would break auth. Keeping `api.quiet-cube.com` costs $0.25/mo and keeps the cookie story intact.

---

## Phase 1 — Code prep on `dev`

Land these on dev so they ride into main with the merge. None of them affect the running dev environment.

- [x] Update `README.md`: replace `cube.nebulouscode.com` (×2) with `quiet-cube.com`.
- [x] Update `docs/ARCHITECTURE.md`: replace `cube.nebulouscode.com` (×2) with `quiet-cube.com`.
- [x] Update `backend/.env.example`: change the `# Production: …` comment for `FRONTEND_URL` to `https://quiet-cube.com`.
- [x] Update `frontend/.env.example`: change the `# Production: …` comment for `VITE_API_BASE_URL` to `https://api.quiet-cube.com`.
- [x] Update `.claude/settings.local.json`: swap the curl allow-list entry from `api.cube.nebulouscode.com` to `api.quiet-cube.com`.
- [x] Decide: `docs/CHANGELOG.md` mentions `cube.nebulouscode.com` (×2). Leave as historical record OR update — your call. Milestone docs under `docs/milestones/*` and `docs/MVP_QA_CHECKLIST.md` are historical; leave them.
- [x] `cargo test` + `npm run type-check` pass.
- [ ] Commit + push to `dev`.

---

## Phase 1.5 — Free up the custom-domain credits on dev

Render bills custom domains per slot. Removing the dev domains before adding the prod ones lets you reuse the same credits instead of paying for both at once. After this phase, dev runs on bare `*.onrender.com` URLs. Dev auth will be cross-site cookies (won't carry), but production hasn't started yet so this is fine — dev is in the same cleanup-state Phase 9 was always going to leave it in.

- [x] In Render → dev frontend service → Custom Domains: remove `cube.nebulouscode.com`.
- [x] In Render → dev backend service → Custom Domains: remove `api.cube.nebulouscode.com`.
- [x] At the registrar / DNS host: leave the existing CNAME records pointing at the (now-empty) Render dev services. We'll repoint them in Phase 8 to redirect to quiet-cube.com.
- [x] Confirm both Render services now show only their bare `*.onrender.com` URL.

## Phase 2 — Turnstile dashboard

- [x] Open the Turnstile site in the Cloudflare dashboard.
- [x] Add hostnames: `quiet-cube.com`, `api.quiet-cube.com`.
- [x] Leave `cube.nebulouscode.com` and `localhost` in the list for now (so dev keeps working until you migrate it).
- [x] No code or env-var changes — site/secret keys stay the same.

---

## Phase 3 — Merge to `main`

- [ ] `git checkout main && git pull`
- [ ] `git merge --no-ff dev` (or fast-forward — your call; `--no-ff` keeps a visible merge commit for the launch).
- [ ] `git push origin main`
- [ ] **Do NOT** trigger the Render redeploy yet — that comes in Phase 4 after the service is configured to track `main`.

---

## Phase 4 — Render: switch deploys to `main`, add custom domains

How you do this depends on whether you want dev to keep auto-deploying:

- **Option A — repurpose existing services** (simpler, dev's auto-deploy stops): change the branch on the existing backend + frontend Render services from `dev` → `main`. Dev pushes no longer trigger a deploy; you manually deploy dev when needed.
- **Option B — clone services for `main`** (keeps dev alive): create new backend + frontend services on Render tracking `main`. Leave the existing dev services alone.

Option A is the path of least resistance for a small project.

- [ ] Switch (or create) the backend service to track `main`. First deploy lands on the bare `*.onrender.com` URL.
- [ ] Switch (or create) the frontend service to track `main`. First deploy lands on the bare `*.onrender.com` URL.
- [ ] Smoke-test the bare URLs:
  - [ ] `curl https://<backend>.onrender.com/api/v1/health` → `{"status":"ok"}`
  - [ ] Open `https://<frontend>.onrender.com` in incognito. Page loads. (Auth won't work yet — that's expected; we haven't pointed it at the backend.)
- [ ] In Render → backend service → Custom Domains, add `api.quiet-cube.com`. Note the CNAME target Render gives you.
- [ ] In Render → frontend service → Custom Domains, add `quiet-cube.com` (and `www.quiet-cube.com` if you bought it / want it to redirect). Note the CNAME target(s).

---

## Phase 5 — DNS

At your registrar (where you bought `quiet-cube.com`):

- [ ] Create CNAME: `api.quiet-cube.com` → the target Render gave for the backend custom domain.
- [ ] Create CNAME/ALIAS: `quiet-cube.com` apex → the target Render gave for the frontend. (Most registrars need an ALIAS/ANAME for apex; if yours only does CNAME, use `www.quiet-cube.com` as the canonical and 301 the apex.)
- [ ] (Optional) `www.quiet-cube.com` → apex, via the registrar or Render.
- [ ] Wait for propagation. `dig api.quiet-cube.com` and `dig quiet-cube.com` should show the new targets within a few minutes; SSL provisioning by Render usually takes 1–5 min after DNS resolves.
- [ ] Verify in browser: both URLs serve over HTTPS without cert warnings.

---

## Phase 6 — Backend / frontend env vars (after DNS is live)

- [ ] Backend service env on Render: set `FRONTEND_URL=https://quiet-cube.com`. **Save** → triggers a redeploy.
- [ ] Frontend service env on Render: set `VITE_API_BASE_URL=https://api.quiet-cube.com`. **Save** → trigger a **manual rebuild** (Vite bakes `VITE_*` at build time; saving the env var alone is not enough).
- [ ] After both redeploys finish: confirm in the frontend's DevTools → Network that requests now go to `api.quiet-cube.com`, not the old hostname.

---

## Phase 7 — Smoke test on `quiet-cube.com`

Run the whole register-to-practice flow against the live site. **Use a real email you control** — Resend will only deliver if the domain is verified, which is `mail.nebulouscode.com` (already set up).

- [ ] Open `https://quiet-cube.com` in incognito. Landing page renders.
- [ ] Click "Create an account". `/register` loads.
- [ ] Register with a fresh email + 8-char password. Confirm Turnstile doesn't block.
- [ ] Verification email arrives within a minute.
- [ ] Enter the 6-digit code. Verify succeeds, lands on `/welcome` then `/practice`.
- [ ] Solve a couple of cards. Streak appears on `/progress`.
- [ ] Sign out → sign back in. Streak preserved.
- [ ] Open `/settings` → "Delete account" → confirm. Account deletion succeeds.
- [ ] Re-register with the **same email** that was just deleted. Streak shows 0 — confirms the [streak-non-zero-after-delete bug](TODO.md) is closed.
- [ ] Delete the test account again for cleanup.

---

## Phase 8 — Redirect old hostname

- [ ] At your DNS host or via a one-line Render redirect rule: 301 `cube.nebulouscode.com` → `https://quiet-cube.com` (catch-all, preserving path).
- [ ] (Optional) 301 `api.cube.nebulouscode.com` → `https://api.quiet-cube.com` if you want to be tidy; not strictly necessary because nothing legitimate hits the old backend URL once the frontend is rebuilt.
- [ ] Test: `curl -sI https://cube.nebulouscode.com/` returns `301` and `Location: https://quiet-cube.com/`.

---

## Phase 9 — Dev environment cleanup

The custom domains were already removed back in Phase 1.5. Remaining cleanup:

- [ ] Update dev frontend env var: `VITE_API_BASE_URL` → the dev backend's bare `*.onrender.com` URL. Trigger a rebuild.
- [ ] In Cloudflare Turnstile, remove `cube.nebulouscode.com` from the hostnames list (keep `localhost` for local dev).
- [ ] Confirm dev frontend on bare onrender.com loads. Auth won't fully work cross-site (cookies won't carry), but that's the cost of the dev cleanup — you noted in `TODO.md` you'd hit onrender.com manually for dev.

---

## Phase 10 — Close out

- [ ] `docs/TODO.md`: check off "Render Main Branch" (line 35) and "Remove API URL Redirect" (line 27 — note: not done literally; we *kept* the redirect for cookie reasons and accepted the $0.25/mo cost. Either check it off with a note, or delete it).
- [ ] `docs/TODO.md`: check off "Dev Render Environment" (line 39).
- [ ] `docs/CHANGELOG.md`: add an entry for the launch under `quiet-cube.com`.
- [ ] Tag the launch commit: `git tag -a v1.0 -m "Public launch on quiet-cube.com" && git push origin v1.0`.
- [ ] Tell someone.

---

## Rollback (if everything goes sideways)

If the cutover breaks and you can't fix forward:

- [ ] At the registrar, remove the `quiet-cube.com` / `api.quiet-cube.com` DNS records (or point them back to `cube.nebulouscode.com` / `api.cube.nebulouscode.com` if the dev services are still alive).
- [ ] In Render, switch the services back to deploying `dev`.
- [ ] In Render, restore `cube.nebulouscode.com` + `api.cube.nebulouscode.com` as custom domains on the dev services.
- [ ] Wait for DNS / cert propagation. Old hostname comes back to life within a few minutes.

The new hostnames are clean adds; there's nothing destructive in the cutover (no DB migrations, no deletions). Worst case is downtime, not data loss.
