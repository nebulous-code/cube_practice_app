<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
</script>

<template>
  <main class="placeholder">
    <div class="card">
      <p class="eyebrow">Milestone 1</p>
      <h1>Cube Practice — scaffolding</h1>
      <p class="body">
        Auth flows are landing this milestone. Practice, cases, and progress views come online
        in milestones 2&ndash;4.
      </p>
      <div class="actions">
        <button class="link" type="button" @click="router.push('/register')">Create an account →</button>
        <button class="link muted" type="button" @click="router.push('/login')">Sign in</button>
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
  margin: 0 0 var(--space-6);
}

.actions {
  display: flex;
  gap: var(--space-4);
  flex-wrap: wrap;
}

.link {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  color: var(--paper-accent);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 3px;
}

.link.muted {
  color: var(--paper-ink-muted);
}
</style>
