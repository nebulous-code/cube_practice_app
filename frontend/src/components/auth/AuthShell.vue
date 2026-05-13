<script setup lang="ts">
import { RouterLink } from 'vue-router'

defineProps<{ onBack?: () => void }>()
</script>

<template>
  <div class="shell">
    <div v-if="onBack" class="back-row">
      <button class="back" type="button" @click="onBack?.()">← Back</button>
    </div>
    <div class="body" :class="{ 'with-back': !!onBack }">
      <slot />
    </div>
    <div v-if="$slots.footer" class="footer">
      <slot name="footer" />
    </div>
    <nav class="legal">
      <RouterLink to="/about">About</RouterLink>
      <span aria-hidden="true">·</span>
      <RouterLink to="/terms">Terms</RouterLink>
      <span aria-hidden="true">·</span>
      <RouterLink to="/privacy">Privacy</RouterLink>
    </nav>
  </div>
</template>

<style scoped>
.shell {
  background: var(--paper-bg);
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  color: var(--paper-ink);
}

.back-row {
  padding: 52px 22px 0;
}

.back {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  cursor: pointer;
}

.body {
  flex: 1;
  padding: 64px 26px 0;
}

.body.with-back {
  padding-top: 20px;
}

.footer {
  padding: 20px 26px 12px;
  text-align: center;
}

.legal {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  padding: 8px 26px 24px;
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--paper-ink-faint);
}

.legal a {
  color: var(--paper-ink-faint);
  text-decoration: none;
}

.legal a:hover {
  color: var(--paper-ink-muted);
  text-decoration: underline;
  text-underline-offset: 2px;
}
</style>
