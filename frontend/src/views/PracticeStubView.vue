<script setup lang="ts">
import { useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()
</script>

<template>
  <main class="stub">
    <div class="card">
      <p class="eyebrow">Practice</p>
      <h1>Hi, {{ auth.user?.display_name ?? 'there' }}.</h1>
      <p class="body">
        The study loop arrives in milestone 3 — pattern → reveal → grade
        with Anki-style spaced repetition. For now, browse all 57 cases
        from the Cases tab.
      </p>
      <button class="primary" type="button" @click="router.push('/cases')">
        Browse cases →
      </button>
    </div>
  </main>
</template>

<style scoped>
.stub {
  min-height: calc(100vh - 90px);
  display: grid;
  place-items: center;
  padding: 48px 22px 40px;
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
  line-height: 1.1;
  color: var(--paper-ink);
  margin: 0 0 var(--space-4);
}

.body {
  font-family: var(--font-serif);
  font-style: italic;
  color: var(--paper-ink-muted);
  line-height: 1.5;
  margin: 0 0 var(--space-5);
}

.primary {
  background: transparent;
  border: 1px solid var(--paper-rule);
  border-radius: 12px;
  padding: 10px 16px;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  color: var(--paper-ink);
  cursor: pointer;
}
</style>
