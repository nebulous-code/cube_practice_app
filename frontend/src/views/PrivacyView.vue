<script setup lang="ts">
import StaticPageView from './StaticPageView.vue'
</script>

<template>
  <StaticPageView eyebrow="Legal" title="Privacy Policy">
    <p class="doc-meta"><strong>Last updated:</strong> May 12, 2026</p>

    <h2>About this policy</h2>
    <p>
      Quiet Cube ("the app", "we", "us") is a hobby/portfolio project that helps users practice Rubik's cube and other puzzle cube algorithms. This policy explains what we collect, why, and what you can do about it. We've tried to write it in plain English. If anything is unclear, email <strong>me@nebulouscode.com</strong>.
    </p>
    <p>
      This is a non-commercial project. We are not a company. We don't sell data, we don't run ads, and we don't share your information with marketing partners.
    </p>

    <h2>What we collect</h2>

    <h3>When you create an account</h3>
    <ul>
      <li><strong>Email address</strong> — used to identify your account, send transactional emails, and contact you about your account</li>
      <li><strong>Display name</strong> — shown in the app interface; you choose it</li>
      <li><strong>Password</strong> — stored only as an Argon2id hash following the <a href="https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html" target="_blank" rel="noopener noreferrer">OWASP password-storage cheatsheet</a>. The plaintext is never written to disk or logs</li>
      <li><strong>Email-verified flag</strong> — whether you've clicked the link we sent you</li>
      <li><strong>Short-lived codes</strong> — temporary tokens for email verification and password reset</li>
      <li><strong>Pending email value</strong> — held during an email-change flow until you confirm the new address</li>
    </ul>

    <h3>Things you do in the app</h3>
    <ul>
      <li><strong>Per-case overrides</strong> — custom algorithms, nicknames, and tags you set for individual cube cases</li>
      <li><strong>Learning state and SM-2 review history</strong> — one row per practice grade so the spaced-repetition system works</li>
      <li><strong>Streak count and last-practice date</strong></li>
      <li><strong>Free-study filter preferences</strong></li>
    </ul>
    <p>
      All of this is scoped to your account and is not visible to other users. There are no public profiles, shared timelines, or comment threads.
    </p>

    <h3>Sessions</h3>
    <p>
      When you log in, we create a server-side session row containing a hashed session token, an expiry, and a revoked flag. This is what lets you sign out of all devices. The session cookie itself is described under "Cookies and local storage" below.
    </p>

    <h3>Account deletion records</h3>
    <p>
      When you delete your account, we keep one row in an <code>account_deletions</code> table containing your original email address and the deletion timestamp. We use this to investigate abuse and to enforce re-registration limits. Everything else tied to your account is purged immediately.
    </p>
    <p>
      If you want this row removed too, email us and we'll handle it case by case.
    </p>

    <h2>What we don't collect</h2>
    <ul>
      <li>We don't store IP addresses in the database, and our application logs don't include them.</li>
      <li>We don't use analytics (no Google Analytics, Plausible, PostHog, or similar).</li>
      <li>We don't use error tracking services like Sentry.</li>
      <li>We don't store birthdays, ages, real names, phone numbers, or any other personal information beyond what's listed above.</li>
      <li>We don't track you across other websites.</li>
    </ul>

    <h2>IP addresses</h2>
    <p>
      Although we don't store IP addresses ourselves, a few things in the request path see them:
    </p>
    <ul>
      <li><strong>Our rate limiter</strong> holds IPs in an in-memory map (keyed on <code>X-Forwarded-For</code>) only as long as the rate-limit window is open, then discards them. They're never persisted.</li>
      <li><strong>Render</strong> (our host) and <strong>Cloudflare</strong> (our CDN) both see client IPs as a normal part of routing HTTP traffic. Their handling is governed by their own privacy policies.</li>
    </ul>

    <h2>Cookies and local storage</h2>

    <h3>Cookies we set</h3>
    <ul>
      <li><strong>Session cookie</strong> — a single cookie carrying a JWT. It is <code>httpOnly</code>, <code>Secure</code>, <code>SameSite=Strict</code>, scoped to <code>/</code>, with a 30-day max age. This is the entire login state.</li>
    </ul>
    <p>
      We do not set any analytics, advertising, or tracking cookies.
    </p>

    <h3>Cookies set by third parties on our behalf</h3>
    <ul>
      <li><strong>Cloudflare</strong> may set bot-management and security cookies such as <code>__cf_bm</code> and <code>cf_clearance</code>. These are set by Cloudflare's edge, not by our application.</li>
      <li><strong>Google reCAPTCHA</strong> sets cookies on the registration page (see "Third-party services" below).</li>
    </ul>

    <h3>Local storage</h3>
    <p>
      We use <code>localStorage</code> only in guest mode. A single key, <code>oll-guest-state</code>, holds your per-case overrides, learning progress, last-practice date, and a small device identifier. It's written with a debounce and stays on your device until you either register an account (at which point it's folded into the new account) or click "Discard guest data".
    </p>

    <h2>Third-party services (subprocessors)</h2>
    <p>
      These services may process your data on our behalf:
    </p>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Service</th>
            <th>Purpose</th>
            <th>What it sees</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><strong>Render</strong></td>
            <td>Application hosting (US East)</td>
            <td>All request traffic, including IPs</td>
          </tr>
          <tr>
            <td><strong>Neon</strong></td>
            <td>PostgreSQL database (US East)</td>
            <td>All data stored in the database</td>
          </tr>
          <tr>
            <td><strong>Cloudflare</strong></td>
            <td>CDN and bot mitigation</td>
            <td>Request metadata including IPs; may set its own cookies</td>
          </tr>
          <tr>
            <td><strong>Resend</strong></td>
            <td>Transactional email delivery</td>
            <td>Your email address and the contents of transactional emails (verification, password reset, etc.)</td>
          </tr>
          <tr>
            <td><strong>Google reCAPTCHA v3</strong></td>
            <td>Bot detection on the registration form</td>
            <td>A token derived from your browser and IP; Google returns a score we use to accept or reject the signup</td>
          </tr>
        </tbody>
      </table>
    </div>
    <p>
      reCAPTCHA is governed by <a href="https://policies.google.com/privacy" target="_blank" rel="noopener noreferrer">Google's Privacy Policy</a> and <a href="https://policies.google.com/terms" target="_blank" rel="noopener noreferrer">Terms of Service</a>.
    </p>

    <h2>Where your data lives</h2>
    <p>
      All application data is stored in the US East region (Neon and Render). If you are outside the United States, using the app means your data will be transferred to and processed in the US.
    </p>

    <h2>How long we keep your data</h2>
    <ul>
      <li><strong>Active account data:</strong> as long as your account exists.</li>
      <li><strong>Verification and password-reset codes:</strong> until used or expired (short-lived, typically minutes to hours).</li>
      <li><strong>Session rows:</strong> until expired or revoked.</li>
      <li><strong>Rate-limiter IP entries:</strong> until the rate-limit window closes (seconds to minutes).</li>
      <li><strong>Database backups:</strong> we rely on Neon's built-in point-in-time restore, which retains a continuous history within Neon's retention window (currently 24 hours on the free tier, longer on paid tiers). Backups are stored by Neon in the same region as the live database.</li>
      <li><strong>After account deletion:</strong> all data is purged immediately, except the single <code>account_deletions</code> row described above.</li>
    </ul>

    <h2>Email</h2>
    <p>
      We currently send only transactional email: email verification, password reset, account-change confirmations, and similar.
    </p>
    <p>
      We may add product-update emails in the future. If we do, they will be opt-in, and you'll be able to unsubscribe.
    </p>

    <h2>Payments</h2>
    <p>
      The app itself does not process payments. The Acknowledgements page links to a GitHub repository where, if you want to support the project, you can use GitHub Sponsors. That flow runs entirely on GitHub and Stripe and never touches our backend.
    </p>

    <h2>Children</h2>
    <p>
      Quiet Cube is not directed to children under 13, and we do not knowingly collect personal information from anyone under 13. If you believe a child under 13 has created an account, email <strong>me@nebulouscode.com</strong> and we will delete it.
    </p>
    <p>
      We do not perform age verification.
    </p>

    <h2>Your rights</h2>
    <p>
      You can do the following at any time:
    </p>
    <ul>
      <li><strong>Access</strong> your data — most of it is visible in the app; email us for anything else.</li>
      <li><strong>Correct</strong> your display name or email from your account settings.</li>
      <li><strong>Delete</strong> your account using the self-serve button in account settings. This purges your data immediately, with the single exception described above.</li>
      <li><strong>Export</strong> your data — email us and we'll send you a copy of what we have.</li>
    </ul>

    <h3>If you are in the European Economic Area, UK, or Switzerland</h3>
    <p>
      The General Data Protection Regulation (GDPR) gives you additional rights, including the right to access, rectify, erase, restrict, port, and object to processing of your personal data. You also have the right to lodge a complaint with your local data protection authority.
    </p>
    <p>
      The legal bases we rely on are: <strong>performance of a contract</strong> (operating the account you signed up for), <strong>legitimate interests</strong> (preventing abuse, securing the service), and <strong>consent</strong> (where you opt in to anything optional in the future).
    </p>

    <h3>If you are in California</h3>
    <p>
      The California Consumer Privacy Act (CCPA) gives you the right to know what personal information we collect, to delete it, and to opt out of any sale or sharing of it. We do not sell or share personal information. To exercise these rights, email <strong>me@nebulouscode.com</strong>.
    </p>

    <h2>Security</h2>
    <p>
      We use Argon2id for password hashing, enforce HTTPS, set restrictive session cookie flags, hash session tokens at rest, and rate-limit authentication endpoints. No system is perfectly secure, but we try to follow current best practices for a project of this size.
    </p>

    <h2>Changes to this policy</h2>
    <p>
      If we change this policy, we'll update the "Last updated" date at the top and, for any material change, notify account holders by email.
    </p>

    <h2>Contact</h2>
    <p>
      <strong>nebulous-code</strong> — <strong>me@nebulouscode.com</strong>
    </p>
  </StaticPageView>
</template>
