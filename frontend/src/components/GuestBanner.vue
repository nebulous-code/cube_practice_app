<script setup lang="ts">
// Persistent guest-mode reminder. Renders inside AppShell above the tab
// bar whenever the auth store is in guest mode and the user hasn't
// suppressed it. Two dismiss paths:
//   - × button = hide for this session only; reappears on next bootstrap.
//   - "Hide until 10 reviews" = persisted to the blob; reappears once
//     the user has graded 10+ cases.
// See docs/milestones/06_guest_mode.md §5 + §12 Q2.

import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

const SUPPRESS_THRESHOLD = 10

const dismissedThisSession = ref(false)

const reviewCount = computed(() => {
  if (!auth.guestState) return 0
  return Object.keys(auth.guestState.progress).length
})

const visible = computed(() => {
  if (!auth.isGuest || !auth.guestState) return false
  if (dismissedThisSession.value) return false
  const suppress = auth.guestState.banner_suppressed_until_reviews
  if (suppress != null && reviewCount.value < suppress) return false
  return true
})

function dismissForSession() {
  dismissedThisSession.value = true
}

function suppressUntilReviews() {
  auth.updateGuestState((s) => {
    s.banner_suppressed_until_reviews = SUPPRESS_THRESHOLD
    s.banner_dismissed_at = new Date().toISOString()
  })
  dismissedThisSession.value = true
}

function goUpgrade() {
  router.push('/upgrade')
}
</script>

<template>
  <Transition name="fade">
    <aside v-if="visible" class="banner" role="status" aria-live="polite">
      <div class="content">
        <p class="copy">
          You're practicing as a guest. Progress is saved on this device only.
        </p>
        <div class="actions">
          <button type="button" class="cta" @click="goUpgrade">
            Save your progress →
          </button>
          <button
            type="button"
            class="suppress"
            @click="suppressUntilReviews"
          >
            Hide until 10 reviews
          </button>
        </div>
      </div>
      <button
        type="button"
        class="dismiss"
        aria-label="Dismiss banner for this session"
        @click="dismissForSession"
      >
        ×
      </button>
    </aside>
  </Transition>
</template>

<style scoped>
.banner {
  position: fixed;
  bottom: calc(env(safe-area-inset-bottom, 0px) + 64px);
  left: 0;
  right: 0;
  z-index: 4;
  background: var(--paper-card);
  border-top: 1px solid var(--paper-rule-faint);
  border-bottom: 1px solid var(--paper-rule-faint);
  padding: 10px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.copy {
  font-family: var(--font-sans);
  font-size: 12px;
  line-height: 1.4;
  color: var(--paper-ink-muted);
  margin: 0;
}

.actions {
  display: flex;
  gap: 12px;
  align-items: baseline;
  flex-wrap: wrap;
}

.cta {
  background: transparent;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 500;
  color: var(--paper-ink);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 3px;
  text-decoration-color: var(--paper-rule);
}

.cta:hover {
  text-decoration-color: var(--paper-ink);
}

.suppress {
  background: transparent;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--paper-ink-faint);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
  text-decoration-color: var(--paper-rule-faint);
}

.suppress:hover {
  color: var(--paper-ink-muted);
}

.dismiss {
  background: transparent;
  border: none;
  padding: 4px 8px;
  font-size: 22px;
  line-height: 1;
  color: var(--paper-ink-faint);
  cursor: pointer;
}

.dismiss:hover {
  color: var(--paper-ink-muted);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 200ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
