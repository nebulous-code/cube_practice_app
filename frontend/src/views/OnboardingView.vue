<script setup lang="ts">
// Three-step onboarding at /welcome. Triggered exactly once via
// VerifyEmailView's success handler when has_seen_onboarding === false.
// Both completion and skip mark the flag on the backend then route to
// /practice. See docs/milestones/05_polish_and_static_pages.md §5.

import { ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const step = ref<1 | 2 | 3>(1)
const finishing = ref(false)

function next() {
  if (step.value === 1) step.value = 2
  else if (step.value === 2) step.value = 3
}

async function finish() {
  if (finishing.value) return
  finishing.value = true
  try {
    await auth.completeOnboarding()
  } catch {
    // Best-effort — if the flag flip fails, we still proceed. Worst case
    // the user sees the onboarding once more on next verification.
  }
  router.push('/practice')
}
</script>

<template>
  <main class="welcome">
    <button type="button" class="skip" :disabled="finishing" @click="finish">
      Skip onboarding
    </button>

    <article v-if="step === 1" class="card">
      <p class="eyebrow">Welcome</p>
      <h1 class="title">All 57, eventually</h1>
      <p class="body">
        Quiet Cube helps you go from one OLL algorithm to all of them,
        without grinding the cases you've already locked in. Bring a solved
        cube — you'll be executing algorithms against it.
      </p>
      <button type="button" class="cta primary" @click="next">Next →</button>
    </article>

    <article v-else-if="step === 2" class="card">
      <p class="eyebrow">The loop</p>
      <h1 class="title">Solve, flip, grade</h1>
      <p class="body">
        Each card shows an OLL case. Execute your algorithm, flip the card,
        compare your cube to the expected pattern. Grade yourself Fail /
        Hard / Good / Easy. The schedule does the rest.
      </p>
      <button type="button" class="cta primary" @click="next">Next →</button>
    </article>

    <article v-else class="card">
      <p class="eyebrow">Make it yours</p>
      <h1 class="title">Your algorithms, your cube</h1>
      <p class="body">
        Don't like the default algorithm for a case? Edit it — and update
        the expected result to match. Quiet Cube doesn't care which alg
        you use, only that you remember the one you chose. No leaderboards,
        no noise, just OLL and your progress.
      </p>
      <button
        type="button"
        class="cta primary"
        :disabled="finishing"
        @click="finish"
      >
        {{ finishing ? 'Setting up…' : 'Got it →' }}
      </button>
    </article>
  </main>
</template>

<style scoped>
.welcome {
  position: relative;
  min-height: 100vh;
  background: var(--paper-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-8) var(--space-5);
}

.skip {
  position: absolute;
  top: 18px;
  right: 18px;
  background: transparent;
  border: none;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-faint);
  cursor: pointer;
  padding: 6px 8px;
  border-bottom: 1px solid var(--paper-rule-faint);
}

.skip:hover {
  color: var(--paper-ink-muted);
  border-bottom-color: var(--paper-rule);
}

.skip:disabled {
  cursor: default;
  opacity: 0.6;
}

.card {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-md);
  padding: var(--space-8) var(--space-6);
  max-width: 440px;
  width: 100%;
  text-align: center;
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.5px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  margin: 0 0 var(--space-3);
}

.title {
  font-family: var(--font-serif);
  font-size: 26px;
  font-weight: 500;
  letter-spacing: -0.4px;
  margin: 0 0 var(--space-3);
  color: var(--paper-ink);
}

.body {
  font-family: var(--font-sans);
  font-size: 14px;
  line-height: 1.6;
  color: var(--paper-ink-muted);
  margin: 0 0 var(--space-5);
}

.cta {
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  padding: 10px 20px;
  border-radius: var(--radius-md);
  cursor: pointer;
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

.cta:disabled {
  opacity: 0.6;
  cursor: default;
}
</style>
