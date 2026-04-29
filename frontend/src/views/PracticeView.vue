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

const dateStr = computed(() =>
  new Date().toLocaleDateString('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
  }),
)

const streakUnit = computed(() => (study.streak.count === 1 ? 'day' : 'days'))
const dueUnit = computed(() => (dueCount.value === 1 ? 'case' : 'cases'))

function startSession() {
  if (!study.startSession()) return
  router.push('/study')
}
</script>

<template>
  <main class="page">
    <!-- Masthead -->
    <header class="masthead">
      <p class="eyebrow">{{ dateStr }}</p>
      <h1 class="title">
        Hi,
        <span class="italic">{{ auth.user?.display_name ?? 'there' }}.</span>
      </h1>
    </header>

    <!-- KPI row — Streak (emphasis) + Due -->
    <section class="kpi-row">
      <div class="kpi emphasis">
        <p class="kpi-label">Streak</p>
        <div class="kpi-row-value">
          <span class="kpi-value">{{ study.streak.count }}</span>
          <span class="kpi-unit">{{ streakUnit }}</span>
        </div>
      </div>
      <div class="kpi">
        <p class="kpi-label">Due</p>
        <div class="kpi-row-value">
          <span class="kpi-value">{{ dueCount }}</span>
          <span class="kpi-unit">{{ dueUnit }}</span>
        </div>
      </div>
    </section>

    <!-- Today's queue / CTA -->
    <section class="queue-card">
      <p class="queue-eyebrow">Today's queue</p>
      <p class="queue-count">{{ dueCount }} {{ dueUnit }}</p>
      <p v-if="dueCount > 0" class="queue-sub">oldest first</p>
      <p v-else class="queue-sub">nothing waiting</p>

      <button
        v-if="dueCount > 0"
        type="button"
        class="primary"
        @click="startSession"
      >
        Begin session →
      </button>
      <button
        v-else
        type="button"
        class="ghost"
        @click="router.push('/cases')"
      >
        Browse cases
      </button>
    </section>

    <p v-if="study.status === 'error'" class="error">
      {{ study.error }}
    </p>

    <p v-if="dueCount === 0" class="empty-hint">
      No cards due. Open a case from the
      <a class="hint-link" @click="router.push('/cases')">Cases tab</a>
      and tap "Start studying" to add it to your queue.
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

.masthead {
  margin-bottom: 18px;
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

.title {
  font-family: var(--font-serif);
  font-size: 36px;
  letter-spacing: -1px;
  line-height: 1.05;
  color: var(--paper-ink);
  margin: 0;
}

.italic {
  font-style: italic;
  color: var(--paper-ink-muted);
}

.kpi-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-bottom: 16px;
}

.kpi {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 12px;
  padding: 12px 14px;
}

.kpi.emphasis {
  background: var(--paper-accent-bg);
  border-color: var(--paper-accent-bg);
}

.kpi-label {
  font-family: var(--font-sans);
  font-size: 9.5px;
  letter-spacing: 1.2px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 6px;
}

.kpi.emphasis .kpi-label {
  color: var(--paper-accent);
}

.kpi-row-value {
  display: flex;
  align-items: baseline;
  gap: 5px;
}

.kpi-value {
  font-family: var(--font-serif);
  font-size: 28px;
  line-height: 1;
  letter-spacing: -0.8px;
  color: var(--paper-ink);
  font-weight: 500;
}

.kpi.emphasis .kpi-value {
  color: var(--paper-accent);
}

.kpi-unit {
  font-family: var(--font-sans);
  font-size: 10px;
  letter-spacing: 0.3px;
  color: var(--paper-ink-muted);
}

.kpi.emphasis .kpi-unit {
  color: var(--paper-accent);
  opacity: 0.8;
}

.queue-card {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border-radius: 18px;
  padding: 22px 22px 20px;
}

.queue-eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.4px;
  text-transform: uppercase;
  opacity: 0.55;
  margin: 0 0 10px;
}

.queue-count {
  font-family: var(--font-serif);
  font-size: 38px;
  line-height: 1;
  letter-spacing: -1px;
  margin: 0 0 4px;
}

.queue-sub {
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 16px;
  opacity: 0.7;
  margin: 0 0 20px;
}

.primary {
  width: 100%;
  background: var(--paper-bg);
  color: var(--paper-ink);
  border: none;
  padding: 14px;
  border-radius: 12px;
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  letter-spacing: 0.2px;
}

.primary:hover {
  opacity: 0.92;
}

.ghost {
  width: 100%;
  background: transparent;
  color: var(--paper-bg);
  border: 1px solid rgba(255, 255, 255, 0.25);
  padding: 14px;
  border-radius: 12px;
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
}

.error {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-error);
  margin-top: 16px;
}

.empty-hint {
  margin-top: 18px;
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 14px;
  color: var(--paper-ink-muted);
  line-height: 1.5;
}

.hint-link {
  color: var(--paper-accent);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}
</style>
