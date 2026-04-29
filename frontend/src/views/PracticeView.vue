<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'
import { useStudyStore } from '@/stores/study'

const router = useRouter()
const auth = useAuthStore()
const study = useStudyStore()

onMounted(() => {
  void study.loadDue()
})

const dueCount = computed(() => study.queue.length)

function startSession() {
  if (!study.startSession()) return
  router.push('/study')
}
</script>

<template>
  <main class="page">
    <header class="head">
      <p class="greeting">
        Hi, {{ auth.user?.display_name ?? 'there' }}.
      </p>
      <div class="streak">
        <p class="streak-label">Streak</p>
        <p class="streak-value">{{ study.streak.count }}</p>
      </div>
    </header>

    <section class="due-card">
      <p class="eyebrow">Due today</p>
      <p class="due-count">{{ dueCount }}</p>

      <button
        v-if="dueCount > 0"
        type="button"
        class="primary"
        @click="startSession"
      >
        Start session
      </button>
      <p v-else class="empty">
        Nothing due. Open the
        <a class="empty-link" @click="router.push('/cases')">Cases tab</a>
        and tap "Start studying" on any case to add it to your queue.
      </p>
    </section>

    <p v-if="study.status === 'error'" class="error">
      {{ study.error }}
    </p>
  </main>
</template>

<style scoped>
.page {
  background: var(--paper-bg);
  min-height: calc(100vh - 90px);
  padding: 36px 22px 40px;
  color: var(--paper-ink);
}

.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 28px;
}

.greeting {
  font-family: var(--font-serif);
  font-size: 26px;
  letter-spacing: -0.5px;
  color: var(--paper-ink);
  margin: 0;
}

.streak {
  text-align: right;
}

.streak-label {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 4px;
}

.streak-value {
  font-family: var(--font-serif);
  font-size: 28px;
  line-height: 1;
  color: var(--paper-ink);
  margin: 0;
}

.due-card {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 14px;
  padding: 24px;
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 8px;
}

.due-count {
  font-family: var(--font-serif);
  font-size: 56px;
  line-height: 1;
  letter-spacing: -1px;
  color: var(--paper-ink);
  margin: 0 0 18px;
}

.primary {
  width: 100%;
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: none;
  border-radius: 12px;
  padding: 14px;
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
}

.primary:hover {
  opacity: 0.92;
}

.empty {
  font-family: var(--font-serif);
  font-style: italic;
  color: var(--paper-ink-muted);
  line-height: 1.5;
  margin: 0;
}

.empty-link {
  color: var(--paper-accent);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.error {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-error);
  margin-top: 16px;
}
</style>
