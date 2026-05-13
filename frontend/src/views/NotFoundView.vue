<script setup lang="ts">
// Catch-all 404. Public route — must not leak any user data. The CTA target
// is auth-aware: signed-in users go back to their dashboard, guests go to
// the login screen. See docs/milestones/05_polish_and_static_pages.md §5.

import { computed } from 'vue'
import { RouterLink } from 'vue-router'

import EmptyState from '@/components/EmptyState.vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()

const cta = computed(() =>
  auth.status === 'authed'
    ? { to: '/practice', label: 'Back to practice' }
    : { to: '/login', label: 'Back to login' },
)
</script>

<template>
  <main class="page">
    <EmptyState>
      <template #title>Nothing here.</template>
      <template #body>The page you're looking for doesn't exist.</template>
      <template #cta>
        <RouterLink :to="cta.to" class="cta">{{ cta.label }}</RouterLink>
      </template>
    </EmptyState>
  </main>
</template>

<style scoped>
.page {
  min-height: 100vh;
  background: var(--paper-bg);
  padding: var(--space-10) var(--space-5);
  display: flex;
  align-items: center;
  justify-content: center;
}

.page :deep(.empty) {
  max-width: 420px;
  width: 100%;
}

.cta {
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  padding: 8px 16px;
  border-radius: var(--radius-md);
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: 1px solid var(--paper-ink);
}

.cta:hover {
  background: var(--paper-accent);
  border-color: var(--paper-accent);
}
</style>
