# Privacy Policy

**Last updated:** May 12, 2026

## About this policy

Quiet Cube ("the app", "we", "us") is a hobby/portfolio project that helps users practice Rubik's cube and other puzzle cube algorithms. This policy explains what we collect, why, and what you can do about it. We've tried to write it in plain English. If anything is unclear, email **me@nebulouscode.com**.

This is a non-commercial project. We are not a company. We don't sell data, we don't run ads, and we don't share your information with marketing partners.

## What we collect

### When you create an account

- **Email address** — used to identify your account, send transactional emails, and contact you about your account
- **Display name** — shown in the app interface; you choose it
- **Password** — stored only as an Argon2id hash following the [OWASP password-storage cheatsheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html). The plaintext is never written to disk or logs
- **Email-verified flag** — whether you've clicked the link we sent you
- **Short-lived codes** — temporary tokens for email verification and password reset
- **Pending email value** — held during an email-change flow until you confirm the new address

### Things you do in the app

- **Per-case overrides** — custom algorithms, nicknames, and tags you set for individual cube cases
- **Learning state and SM-2 review history** — one row per practice grade so the spaced-repetition system works
- **Streak count and last-practice date**
- **Free-study filter preferences**

All of this is scoped to your account and is not visible to other users. There are no public profiles, shared timelines, or comment threads.

### Sessions

When you log in, we create a server-side session row containing a hashed session token, an expiry, and a revoked flag. This is what lets you sign out of all devices. The session cookie itself is described under "Cookies and local storage" below.

### Account deletion records

When you delete your account, we keep one row in an `account_deletions` table containing your original email address and the deletion timestamp. We use this to investigate abuse and to enforce re-registration limits. Everything else tied to your account is purged immediately.

If you want this row removed too, email us and we'll handle it case by case.

## What we don't collect

- We don't store IP addresses in the database, and our application logs don't include them.
- We don't use analytics (no Google Analytics, Plausible, PostHog, or similar).
- We don't use error tracking services like Sentry.
- We don't store birthdays, ages, real names, phone numbers, or any other personal information beyond what's listed above.
- We don't track you across other websites.

## IP addresses

Although we don't store IP addresses ourselves, a few things in the request path see them:

- **Our rate limiter** holds IPs in an in-memory map (keyed on `X-Forwarded-For`) only as long as the rate-limit window is open, then discards them. They're never persisted.
- **Render** (our host) and **Cloudflare** (our CDN) both see client IPs as a normal part of routing HTTP traffic. Their handling is governed by their own privacy policies.

## Cookies and local storage

### Cookies we set

- **Session cookie** — a single cookie carrying a JWT. It is `httpOnly`, `Secure`, `SameSite=Strict`, scoped to `/`, with a 30-day max age. This is the entire login state.

We do not set any analytics, advertising, or tracking cookies.

### Cookies set by third parties on our behalf

- **Cloudflare** may set bot-management and security cookies such as `__cf_bm` and `cf_clearance`. These are set by Cloudflare's edge, not by our application. The same applies to the Turnstile widget on the registration page (see "Third-party services" below).

### Local storage

We use `localStorage` only in guest mode. A single key, `oll-guest-state`, holds your per-case overrides, learning progress, last-practice date, and a small device identifier. It's written with a debounce and stays on your device until you either register an account (at which point it's folded into the new account) or click "Discard guest data".

## Third-party services (subprocessors)

These services may process your data on our behalf:

| Service | Purpose | What it sees |
|---|---|---|
| **Render** | Application hosting (US East) | All request traffic, including IPs |
| **Neon** | PostgreSQL database (US East) | All data stored in the database |
| **Cloudflare** | CDN, bot mitigation, and Turnstile (registration form anti-bot) | Request metadata including IPs; may set its own cookies. For Turnstile: a token derived from your browser and IP, returned to us as pass/fail |
| **Resend** | Transactional email delivery | Your email address and the contents of transactional emails (verification, password reset, etc.) |

Cloudflare Turnstile is governed by [Cloudflare's Privacy Policy](https://www.cloudflare.com/privacypolicy/).

## Where your data lives

All application data is stored in the US East region (Neon and Render). If you are outside the United States, using the app means your data will be transferred to and processed in the US.

## How long we keep your data

- **Active account data:** as long as your account exists.
- **Verification and password-reset codes:** until used or expired (short-lived, typically minutes to hours).
- **Session rows:** until expired or revoked.
- **Rate-limiter IP entries:** until the rate-limit window closes (seconds to minutes).
- **Database backups:** we rely on Neon's built-in point-in-time restore, which retains a continuous history within Neon's retention window (currently 24 hours on the free tier, longer on paid tiers). Backups are stored by Neon in the same region as the live database.
- **After account deletion:** all data is purged immediately, except the single `account_deletions` row described above.

## Email

We currently send only transactional email: email verification, password reset, account-change confirmations, and similar.

We may add product-update emails in the future. If we do, they will be opt-in, and you'll be able to unsubscribe.

## Payments

The app itself does not process payments. The Acknowledgements page links to a GitHub repository where, if you want to support the project, you can use GitHub Sponsors. That flow runs entirely on GitHub and Stripe and never touches our backend.

## Children

Quiet Cube is not directed to children under 13, and we do not knowingly collect personal information from anyone under 13. If you believe a child under 13 has created an account, email **me@nebulouscode.com** and we will delete it.

We do not perform age verification.

## Your rights

You can do the following at any time:

- **Access** your data — most of it is visible in the app; email us for anything else.
- **Correct** your display name or email from your account settings.
- **Delete** your account using the self-serve button in account settings. This purges your data immediately, with the single exception described above.
- **Export** your data — email us and we'll send you a copy of what we have.

### If you are in the European Economic Area, UK, or Switzerland

The General Data Protection Regulation (GDPR) gives you additional rights, including the right to access, rectify, erase, restrict, port, and object to processing of your personal data. You also have the right to lodge a complaint with your local data protection authority.

The legal bases we rely on are: **performance of a contract** (operating the account you signed up for), **legitimate interests** (preventing abuse, securing the service), and **consent** (where you opt in to anything optional in the future).

### If you are in California

The California Consumer Privacy Act (CCPA) gives you the right to know what personal information we collect, to delete it, and to opt out of any sale or sharing of it. We do not sell or share personal information. To exercise these rights, email **me@nebulouscode.com**.

## Security

We use Argon2id for password hashing, enforce HTTPS, set restrictive session cookie flags, hash session tokens at rest, and rate-limit authentication endpoints. No system is perfectly secure, but we try to follow current best practices for a project of this size.

## Changes to this policy

If we change this policy, we'll update the "Last updated" date at the top and, for any material change, notify account holders by email.

## Contact

**nebulous-code** — **me@nebulouscode.com**
