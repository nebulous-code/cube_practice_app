<script setup lang="ts">
// Public landing page at `/`. Authed (and guest-mode) visitors are
// redirected to /practice by the router guard, so this view only renders
// for fully-anon visitors.
//
// Copy is placeholder — see docs/TODO.md "Landing page copy" for the swap
// point. Layout follows docs/milestones/05_polish_and_static_pages.md §5.
// M6 swapped the primary hero CTA from "Sign in" to "Continue as guest"
// per the user's clarification on guest_mode_design_doc Q-B.

import { RouterLink, useRouter } from 'vue-router'

import LogoMark from '@/components/auth/LogoMark.vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

function startAsGuest() {
  auth.startGuestMode()
  router.push('/welcome')
}
</script>

<template>
  <div class="landing">
    <header class="topbar">
      <RouterLink to="/" class="brand">Quiet Cube</RouterLink>
      <RouterLink to="/login" class="signin-link">Sign in</RouterLink>
    </header>

    <main>
      <section class="hero">
        <div class="logo">
          <LogoMark :size="72" />
        </div>
        <h1 class="title">Quiet Cube</h1>
        <p class="tag">a quiet place to drill</p>
        <p class="lede">
          Spaced repetition for Rubik's cube algorithms. Build muscle memory for
          the cases you don't yet know, and keep the ones you do sharp.
        </p>
        <div class="cta-row">
          <button type="button" class="cta primary" @click="startAsGuest">
            Continue as guest →
          </button>
          <RouterLink to="/register" class="cta secondary">Create an account</RouterLink>
        </div>
      </section>

      <section class="features">
        <h2 class="section-title">What you get</h2>
        <ul>
          <li>All 57 OLL cases ready out of the box</li>
          <li>Anki-style SM-2 schedules each case for you</li>
          <li>Free study any case, any time</li>
          <li>Track your streak and what's due today</li>
        </ul>
      </section>

      <section class="how">
        <h2 class="section-title">How it works</h2>
        <ol>
          <li>Pick a case to drill, or let the schedule pick for you.</li>
          <li>See the pattern, recall the algorithm, then grade yourself.</li>
          <li>The schedule decides when each case comes back around.</li>
        </ol>
      </section>

      <section class="closing">
        <RouterLink to="/login" class="cta primary">Sign in →</RouterLink>
      </section>
    </main>

    <footer class="footer">
      <RouterLink to="/about">About</RouterLink>
      <span aria-hidden="true">·</span>
      <RouterLink to="/terms">Terms</RouterLink>
      <span aria-hidden="true">·</span>
      <RouterLink to="/privacy">Privacy</RouterLink>
      <span aria-hidden="true">·</span>
      <RouterLink to="/acknowledgements">Acknowledgements</RouterLink>
    </footer>
  </div>
</template>

<style scoped>
.landing {
  min-height: 100vh;
  background: var(--paper-bg);
  color: var(--paper-ink);
  display: flex;
  flex-direction: column;
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--paper-rule-faint);
}

.brand {
  font-family: var(--font-serif);
  font-size: 18px;
  font-weight: 500;
  color: var(--paper-ink);
  text-decoration: none;
  letter-spacing: -0.2px;
}

.signin-link {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  text-decoration: none;
  border-bottom: 1px solid var(--paper-rule);
  padding-bottom: 1px;
}

.signin-link:hover {
  color: var(--paper-ink);
  border-bottom-color: var(--paper-ink);
}

main {
  flex: 1;
  padding: var(--space-8) var(--space-5);
  max-width: 560px;
  margin: 0 auto;
  width: 100%;
}

.hero {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-6) 0 var(--space-8);
}

.logo {
  margin-bottom: var(--space-3);
}

.title {
  font-family: var(--font-serif);
  font-size: 34px;
  font-weight: 500;
  letter-spacing: -0.6px;
  margin: 0;
}

.tag {
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 15px;
  color: var(--paper-ink-faint);
  margin: 0 0 var(--space-3);
}

.lede {
  font-family: var(--font-sans);
  font-size: 15px;
  line-height: 1.55;
  color: var(--paper-ink-muted);
  margin: 0 0 var(--space-5);
  max-width: 40ch;
}

.cta-row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  justify-content: center;
}

.cta {
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  padding: 10px 18px;
  border-radius: var(--radius-md);
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
  cursor: pointer;
}

button.cta {
  appearance: none;
}

.cta.primary {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: 1px solid var(--paper-ink);
}

.cta.primary:hover {
  background: var(--paper-accent);
  border-color: var(--paper-accent);
}

.cta.secondary {
  background: transparent;
  color: var(--paper-ink);
  border: 1px solid var(--paper-rule);
}

.cta.secondary:hover {
  border-color: var(--paper-ink);
}

.section-title {
  font-family: var(--font-serif);
  font-style: italic;
  font-weight: 500;
  font-size: 18px;
  margin: 0 0 var(--space-3);
  color: var(--paper-ink);
}

.features,
.how {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-md);
  padding: var(--space-5);
  margin-bottom: var(--space-5);
}

.features ul,
.how ol {
  margin: 0;
  padding-left: var(--space-5);
  font-family: var(--font-sans);
  font-size: 14px;
  line-height: 1.7;
  color: var(--paper-ink-muted);
}

.features ul {
  list-style: '• ';
  padding-left: var(--space-3);
}

.features li::marker {
  color: var(--paper-ink-faint);
}

.closing {
  text-align: center;
  padding: var(--space-6) 0;
}

.footer {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-5);
  border-top: 1px solid var(--paper-rule-faint);
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-muted);
}

.footer a {
  color: var(--paper-ink-muted);
  text-decoration: none;
}

.footer a:hover {
  color: var(--paper-ink);
}

@media (min-width: 600px) {
  main {
    padding: var(--space-10) var(--space-5);
  }
  .title {
    font-size: 40px;
  }
}
</style>
