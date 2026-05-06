<script setup lang="ts">
import { RouterView, useRouter } from 'vue-router'

import GuestBanner from '@/components/GuestBanner.vue'
import PendingEmailBanner from '@/components/PendingEmailBanner.vue'
import TabBar from '@/components/TabBar.vue'

const router = useRouter()

function goSettings() {
  router.push('/settings')
}
</script>

<template>
  <div class="shell">
    <PendingEmailBanner />
    <button
      type="button"
      class="settings-button"
      aria-label="Settings"
      @click="goSettings"
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="3" />
        <path
          d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.9.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.9 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.9-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.9V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"
        />
      </svg>
    </button>

    <main class="content">
      <RouterView />
    </main>
    <GuestBanner />
    <TabBar />
  </div>
</template>

<style scoped>
.shell {
  background: var(--paper-bg);
  min-height: 100vh;
  color: var(--paper-ink);
  position: relative;
}

.settings-button {
  position: fixed;
  top: 18px;
  right: 18px;
  z-index: 4;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  color: var(--paper-ink-muted);
  cursor: pointer;
  display: grid;
  place-items: center;
  padding: 0;
}

.settings-button:hover {
  color: var(--paper-ink);
  border-color: var(--paper-rule);
}

.settings-button svg {
  width: 18px;
  height: 18px;
}

.content {
  min-height: 100vh;
}
</style>
