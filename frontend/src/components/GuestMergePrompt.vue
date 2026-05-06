<script setup lang="ts">
// One-time prompt rendered when an authed user signs in on a device
// that already has a guest-mode blob in localStorage. Two outcomes:
//   - Merge: POST /auth/merge-guest-state, clear the blob, dismiss.
//   - Discard: clear the blob with no API call, dismiss.
// Either way the prompt never returns on this device — the blob is
// gone after the user picks.
//
// See docs/milestones/06_guest_mode.md §6.

import { computed, ref } from 'vue'

import { ApiError } from '@/api/client'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()

const submitting = ref(false)
const error = ref<string | null>(null)

const visible = computed(() => auth.pendingMergePrompt !== null)

const reviewCount = computed(() => {
  const blob = auth.pendingMergePrompt
  if (!blob) return 0
  return Object.keys(blob.progress).length
})

const settingsCount = computed(() => {
  const blob = auth.pendingMergePrompt
  if (!blob) return 0
  return Object.keys(blob.settings).length
})

async function onMerge() {
  if (submitting.value) return
  submitting.value = true
  error.value = null
  try {
    await auth.mergeGuestState()
  } catch (err) {
    error.value =
      err instanceof ApiError ? err.message : 'Merge failed — try again.'
  } finally {
    submitting.value = false
  }
}

function onDiscard() {
  auth.discardGuestState()
}
</script>

<template>
  <Transition name="fade">
    <div v-if="visible" class="overlay" role="dialog" aria-modal="true">
      <div class="card">
        <p class="eyebrow">Guest data found</p>
        <h2 class="title">Merge into this account?</h2>
        <p class="body">
          We found practice data on this device from a guest session
          <span v-if="reviewCount > 0">
            — {{ reviewCount }} reviewed
            {{ reviewCount === 1 ? 'case' : 'cases' }}<span v-if="settingsCount > 0"
              >, {{ settingsCount }} customized</span
            >.
          </span>
          <span v-else>.</span>
          Merge it in and we'll keep whichever schedule is further along.
        </p>

        <p v-if="error" class="error">{{ error }}</p>

        <div class="actions">
          <button
            type="button"
            class="primary"
            :disabled="submitting"
            @click="onMerge"
          >
            {{ submitting ? 'Merging…' : 'Merge into this account' }}
          </button>
          <button
            type="button"
            class="ghost"
            :disabled="submitting"
            @click="onDiscard"
          >
            Discard guest data
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 10;
  background: rgba(31, 27, 22, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.card {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-md);
  padding: 24px;
  max-width: 420px;
  width: 100%;
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.5px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  margin: 0 0 8px;
}

.title {
  font-family: var(--font-serif);
  font-size: 22px;
  font-weight: 500;
  letter-spacing: -0.4px;
  margin: 0 0 12px;
  color: var(--paper-ink);
}

.body {
  font-family: var(--font-sans);
  font-size: 14px;
  line-height: 1.55;
  color: var(--paper-ink-muted);
  margin: 0 0 16px;
}

.error {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-error);
  margin: 0 0 12px;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.primary,
.ghost {
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  padding: 10px 16px;
  border-radius: var(--radius-md);
  cursor: pointer;
}

.primary {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: 1px solid var(--paper-ink);
}

.primary:hover {
  background: var(--paper-accent);
  border-color: var(--paper-accent);
}

.ghost {
  background: transparent;
  color: var(--paper-ink-muted);
  border: 1px solid var(--paper-rule);
}

.ghost:hover {
  color: var(--paper-ink);
  border-color: var(--paper-ink);
}

.primary:disabled,
.ghost:disabled {
  opacity: 0.6;
  cursor: default;
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
