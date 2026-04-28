<script setup lang="ts">
import { useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

async function onSignOut() {
  await auth.logout()
  router.push('/login')
}
</script>

<template>
  <main class="placeholder">
    <div class="card">
      <p class="eyebrow">Milestone 1</p>
      <h1>Hi, {{ auth.user?.display_name ?? 'there' }}.</h1>
      <p class="body">
        You're signed in. Practice, cases, and progress views come online in milestones
        2&ndash;4.
      </p>
      <p v-if="auth.user" class="meta">
        Signed in as <span class="mono">{{ auth.user.email }}</span>
        <span v-if="auth.user.pending_email">
          &middot; pending change to <span class="mono">{{ auth.user.pending_email }}</span>
        </span>
      </p>
      <div class="actions">
        <button class="signout" type="button" @click="onSignOut">Sign out</button>
      </div>
    </div>
  </main>
</template>

<style scoped>
.placeholder {
  min-height: 100vh;
  display: grid;
  place-items: center;
  padding: var(--space-8) var(--space-6);
}

.card {
  max-width: 480px;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-lg);
  padding: var(--space-8);
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 var(--space-3);
}

h1 {
  font-family: var(--font-serif);
  font-size: 30px;
  letter-spacing: -0.6px;
  margin: 0 0 var(--space-4);
  color: var(--paper-ink);
  line-height: 1.1;
}

.body {
  font-family: var(--font-serif);
  font-style: italic;
  color: var(--paper-ink-muted);
  line-height: 1.5;
  margin: 0 0 var(--space-4);
}

.meta {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-faint);
  margin: 0 0 var(--space-6);
}

.mono {
  font-family: var(--font-mono);
  color: var(--paper-ink-muted);
}

.actions {
  display: flex;
  gap: var(--space-4);
}

.signout {
  background: transparent;
  color: var(--paper-error);
  border: 1px solid var(--paper-rule);
  border-radius: 12px;
  padding: 10px 16px;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}
</style>
