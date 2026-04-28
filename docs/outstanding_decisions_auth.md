# Outstanding Decisions — Auth

Differences between the auth mockups (`initial_design/src/screen-auth.jsx`, `auth-ui.jsx`, `app.jsx`) and the design doc, with a proposed resolution for each.

The original differences (16 below) have been folded into the spec per your responses. New items raised during implementation are listed here at the top.

---

## New questions raised while updating the spec

A. **Pending-email column.** Your response on item 14 said "allow normal use after email change, lock out only if logs out without verifying." Implementing that literally creates a typo-lockout problem: if the user mistypes their new email, they can't log in once they sign out (verification code went to the typo'd address; old email is gone). I changed the spec to use a `pending_email` column instead — the email *only* swaps in `users.email` after the new code is verified. The user's experience matches what you described (banner, no immediate lockout) but typo recovery still works because the original email keeps logging in. Confirm or push back.
> Response: Sounds good.

B. **Verify-email endpoint accepts both authenticated and unauthenticated callers.** During registration the user isn't logged in yet. During an email change they are. The single endpoint dispatches based on auth state: unauthenticated → initial verification + auto-login; authenticated → promote `pending_email` and stay logged in. Same for `POST /auth/resend-verification`. Marked `Auth: Optional` in the API table. Confirm the dual-mode is fine, or split into `POST /auth/verify-email` + `POST /auth/verify-email-change`.
> Response: that works for me.

C. **Splash timing.** Spec says "minimum 800ms display, maximum until `/auth/me` resolves." If the request finishes in 100ms, splash holds at 800ms; if it takes 2s, splash shows for 2s. Confirm this is the right default — easy to tune later.
> Response: yes that's fine. We'll tune later

D. **Sign-out-everywhere safety.** Some apps require the user to re-enter their password before issuing a global revoke (defense against an attacker who has temporary access to a logged-in device). Spec currently doesn't require it. Default proposal: skip the password gate for MVP, add it post-MVP if it becomes a concern. Flag it if you'd rather have it from day one.
> Response: Worth doing from day one. It's not that difficult.

**Resolved.** `POST /auth/sign-out-all` now requires `{ current_password }` in the body. Backend verifies with argon2 before revoking; wrong password returns 401 and touches nothing. Spec §5 "Sign Out Everywhere" and §6 API table updated.

E. **Login response shape.** `GET /auth/me` now returns `{ id, email, display_name, pending_email, email_verified, streak_count, last_practice_date }`. Settings and the streak KPI both rely on this. Confirm the field set, or trim/expand.
> Response: I think it'd be worth separating auth from stats. Since this is a stat focused app there may be more metadata we don't want associated to the user's auth/PII

**Resolved.** Stats split out of `/auth/me`.
- `GET /auth/me` → identity only: `{ id, email, display_name, pending_email, email_verified }`.
- `GET /progress` → app metrics: `{ due_today, learning, mastered, not_started, streak_count, last_practice_date }`. Future stats fields (review counts, accuracy, per-group breakdowns) land here without polluting the auth surface.
- Schema unchanged — `streak_count` and `last_practice_date` still live on the `users` table; this is purely a response-shape decision so PII and metrics stay on disjoint endpoints.
- Frontend impact: `progressStore` (already in §8) is the source of truth for streak; `authStore` no longer carries it.

---

## Original differences

1. **Guest mode is in the mockup but not in the spec.** The login screen has a "Continue as guest" button, and there's a `GuestUpgradeScreen` that migrates a guest's local progress into a new account on sign-up. The earlier decision was "full auth from day one, no guest mode." Pick one.
   - **Proposed:** drop guest mode from the auth flow for MVP. Reasons: avoids two persistence backends (localStorage + server), avoids progress-migration logic, and matches the earlier decision. Easy to add later if you miss it. If you'd rather keep it, the spec needs a section on guest data shape and the upgrade-to-account migration endpoint.
> Response: I'd like you to spec out the guest mode as a separate design doc. It will be the last thing we handle in the MVP. But I want it considered from the beginning so it doesn't feel tacked on at the end

2. **Email verification is a 6-digit code in the mockup, a link in the spec.** Mockup has six single-character boxes; spec says a clickable link in the verification email. These are different flows.
   - **Proposed:** switch the spec to 6-digit codes. Code is friendlier on mobile (no app-switching), simpler email content, and the mockup is already built for it. Schema change: rename `users.verification_token` to `verification_code` (still TEXT, holds 6 digits) and add `verification_code_expires TIMESTAMPTZ` (10 min).
> Response: Great update the spec

3. **Password reset is also a 6-digit code in the mockup.** Same pattern as #2 — spec says token-in-link, mockup uses a 6-digit code.
   - **Proposed:** match #2. Rename `users.reset_token` → `reset_code`, keep `reset_token_expires` (rename to `reset_code_expires`). 1-hour TTL stays.
> Response: great update the spec

4. **Display name field on registration; users table has no name column.** Mockup captures "Display name" at signup and shows it in the avatar/settings.
   - **Proposed:** add `display_name TEXT NOT NULL` to the `users` table. Returned by `GET /auth/me`. Required at registration.
> Response: add a display name to the table in the spec.

5. **Settings screen edits display name and email; no profile-update endpoint in the spec.** "Save changes" is wired up in the mockup.
   - **Proposed:** add `PATCH /auth/me` accepting `{ display_name?, email? }`. If `email` changes, set `email_verified = false`, generate a new `verification_code`, send a new verification email, and lock the new email out of login until verified (or keep them logged in but flag the address as unverified — pick a UX).
> Response: Add this to the spec

6. **"Change password" form in settings; spec only covers forgot/reset.** The mockup has a current + new + confirm form for a logged-in user.
   - **Proposed:** add `POST /auth/change-password` that takes `{ current_password, new_password }`, verifies the current with argon2, hashes and stores the new. Doesn't touch the JWT cookie — the user stays logged in.
> Response: add to spec

7. **"Sign out everywhere" in the security section; no endpoint in the spec.** The `sessions` table is mentioned in the spec for revocation, but no API uses it.
   - **Proposed:** add `POST /auth/sign-out-all` that revokes every session row for the current user (sets `revoked = true`). The current request's session row is also revoked, so the user is signed out on this device too — the UI should redirect to login afterward.
> Response: add to spec

8. **"Resend code" link on the verify screen; no resend endpoint in the spec.**
   - **Proposed:** add `POST /auth/resend-verification`. Generates a fresh `verification_code`, replaces the stored one, and emails it. Rate-limit to one per minute per user. Same pattern can apply to password reset (`POST /auth/forgot-password` is already idempotent — calling it again just sends a fresh code).
> Response: Add to spec

9. **Two-step onboarding screen after verification; not in the spec.** A small intro ("Practice OLL with intention" / "Weakest cases come first") that runs once after first verification and then never again.
   - **Proposed:** keep it in MVP. Frontend-only — no backend changes. Track "has seen onboarding" in `localStorage`; if it's missing on first login, show onboarding once. (Or add a `users.onboarded_at TIMESTAMPTZ` if you want it cross-device — slightly cleaner; small cost.)
> Response: I will have the designer build this flow out once we're further along. For now it's a non-blocker.

10. **reCAPTCHA is in the spec but not in the register mockup.** The spec mandates reCAPTCHA on registration; the mockup's register form doesn't render any captcha widget.
    - **Proposed:** add reCAPTCHA v3 (invisible — no UI, just a token submitted with the form). Keeps the mockup's clean look and still satisfies the spec's bot-prevention requirement. Backend already verifies the token via Google's API.
> Response: I'd like reCAPTCHA to be invisible which is why it's not in spec

11. **Splash screen with a 1.4s timer; not in the spec.** Logo + tagline that auto-advances to login.
    - **Proposed:** keep. Cosmetic, no backend impact. Consider shortening to ~800ms — 1.4s feels long for a returning user.
> Response: It's a place holder for initial loading times that may incur from the db. Redusing it to 800 ms or when the backend gets back to us is fine. Add this to the spec

12. **About / Terms of Service / Privacy Policy / Acknowledgements links; spec doesn't mention these.** They show up on login, in settings, and at registration ("By creating an account you agree to our Terms…").
    - **Proposed:** static pages bundled with the frontend (no API). Mark Terms / Privacy as content-required-before-launch. Acknowledgements can auto-generate from `package.json` / `Cargo.toml` licenses post-MVP.
> Response: Update the spec to call these out. These will just be static pages on the front end. TODO: Get terms and privacy policy made. 

13. **Verification flow auto-logs the user in; spec implies a separate login step after verification.** Mockup goes register → enter code → onboarding → app. Spec reads "User can now log in" after verify, suggesting the user is bounced back to the login form.
    - **Proposed:** match the mockup. The verify endpoint, on success, sets the JWT cookie and returns 200 — user lands directly in onboarding/app. Removes a needless step.
> Response: Update spec to match mockup. There's nothing sensitive that needs high level of security here.

14. **Email change requires re-verification.** Implied by item #5 — calling out separately because it's easy to miss. Until the new email is verified, the user is logged in but their `email_verified` flag is `false`. Decide whether un-verified users can keep using the app or are locked out until they verify.
    - **Proposed:** allow normal use after an email change but show a banner ("Verify your new email to keep your account active") with a resend button. Lock out only if the user logs out and back in without verifying.
> Response: sounds good update the spec.

15. **Splash / unauthenticated route guard.** The mockup uses an in-memory `authStage` state machine; in Vue this becomes a router guard. Worth confirming the contract: any unauthenticated request to a protected route redirects to `/login`; any authenticated request to `/login` redirects to `/`.
    - **Proposed:** that contract, plus a `next` query param so `/login?next=/cases/12` returns the user to where they came from after signing in.
> Response: that sounds good. This was beyond the scope of the mockup. Update the spec.

