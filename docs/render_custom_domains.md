# Render Custom Domains + TLS

How to move `cube-frontend.onrender.com` and `cube-backend.onrender.com` (or whatever Render named them) onto `cube.nebulouscode.com` and `api.cube.nebulouscode.com`. Render auto-provisions TLS via Let's Encrypt — no cert work needed.

---

## Why bother

Two things break or get awkward as long as you're on `*.onrender.com`:

1. **CORS.** Backend's `FRONTEND_URL` env var is the only origin allowed. Each time Render generates a new preview URL or you change service names, you have to chase the env var.
2. **Cookies.** `onrender.com` is on the [Public Suffix List](https://publicsuffix.org/), so `cube-frontend.onrender.com` and `cube-backend.onrender.com` are *different sites* for the browser's same-site checks. Our `SameSite=Strict` session cookie won't travel between them, which breaks any flow that needs the session cookie (verify-email auto-login, `GET /auth/me`, login, settings, sign-out, etc.). Once `cube.nebulouscode.com` and `api.cube.nebulouscode.com` are live they share `nebulouscode.com` as registrable domain — `SameSite=Strict` works without changes.

Register works without this fix because it doesn't set a cookie. Verify-email is the first endpoint that does.

---

## Prereqs

- A custom domain at your DNS provider (you have `nebulouscode.com`).
- Both Render services already deployed and healthy.
- ~5 min of work + up to ~30 min of DNS propagation.

---

## Step 1 — Backend: api.cube.nebulouscode.com

In the Render dashboard:

1. Open your backend service (`cube-backend` or whatever it's named).
2. **Settings → Custom Domains → Add Custom Domain**.
3. Enter `api.cube.nebulouscode.com`. Render shows a CNAME target like `cube-backend.onrender.com.` (note the trailing dot).
4. Leave that page open.

At your DNS provider for `nebulouscode.com`:

5. Add a `CNAME` record:
   - **Name/Host:** `api.cube`
   - **Target/Value:** the Render-provided value (e.g. `cube-backend.onrender.com`)
   - **TTL:** default is fine
   - Save.

Back in Render:

6. Click **Verify**. Status flips through `Pending DNS verification` → `Verifying TLS certificate` → `Issued`. Cert provisions in ~30s once DNS resolves.

---

## Step 2 — Frontend: cube.nebulouscode.com

Same flow on the static site:

1. Open your frontend service (`cube-frontend`).
2. **Settings → Custom Domains → Add Custom Domain** → `cube.nebulouscode.com`.
3. Copy the CNAME target.

At your DNS provider:

4. `CNAME` record, **Name/Host:** `cube`, **Target:** the Render value. Save.

Back in Render:

5. Click **Verify**. Wait for `Issued`.

---

## Step 3 — Update env vars

Once both domains are issued:

**Backend service (Environment tab):**

```
FRONTEND_URL=https://cube.nebulouscode.com
```

**Frontend service (Environment tab):**

```
VITE_API_BASE_URL=https://api.cube.nebulouscode.com
```

Both changes trigger redeploys automatically.

`VITE_*` vars on the frontend are baked at build time, so the redeploy is required for the new value to ship.

---

## Step 4 — Verify

1. `https://api.cube.nebulouscode.com/api/v1/health` → `{"status":"ok"}`
2. `https://cube.nebulouscode.com` loads the placeholder dashboard.
3. Open devtools network tab on `cube.nebulouscode.com`, hit register — the POST to `api.cube.nebulouscode.com/api/v1/auth/register` should succeed.
4. Once verify-email is wired (next slice): registering should set a `Set-Cookie: cube_session=…` header that the browser stores, and the next page load should authenticate.

---

## Optional — drop the onrender.com URLs

You can leave the original `*.onrender.com` URLs active or delete them. They'll keep working alongside the custom domain. Most people leave them for emergency access.

---

## TLS notes

- Render uses Let's Encrypt with auto-renewal. Nothing to do.
- HTTP requests redirect to HTTPS automatically on custom domains.
- Wildcard certs aren't needed — Render issues a separate cert per custom domain.

---

## If something stalls

- Cert stuck in `Verifying`: 99% of the time this is DNS propagation. `dig api.cube.nebulouscode.com` (or `nslookup`) — if it doesn't resolve to the Render target yet, wait. CNAMEs typically propagate in 5–30 min; some registrars take longer.
- Cert stuck and DNS resolves correctly: check that you didn't double-suffix the CNAME name (entered `api.cube.nebulouscode.com` when the registrar already appends the apex — should be just `api.cube`).
- Backend returns 502 after switch: probably a leftover `FRONTEND_URL` referencing the old `.onrender.com` value. Update the env var.
