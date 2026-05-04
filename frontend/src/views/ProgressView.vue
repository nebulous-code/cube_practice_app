<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import CaseStatePip from '@/components/CaseStatePip.vue'
import { type Case, type CaseState, useCasesStore } from '@/stores/cases'
import { useProgressStore } from '@/stores/progress'

const router = useRouter()
const cases = useCasesStore()
const progress = useProgressStore()

type Filter = 'all' | CaseState

const filter = ref<Filter>('all')

onMounted(() => {
  void cases.ensureLoaded()
  void progress.ensureLoaded()
})

const TIER1_LABELS: Record<string, string> = {
  '*': 'Dot',
  L: 'L',
  '-': 'Line',
  '+': 'Cross',
}

const STATE_LABELS: Record<CaseState, string> = {
  not_started: 'Not started',
  learning: 'Learning',
  due: 'Due',
  mastered: 'Mastered',
}

const counts = computed(() => progress.summary?.summary ?? null)

const filtered = computed<Case[]>(() => {
  const list = cases.list.slice().sort((a, b) => a.case_number - b.case_number)
  if (filter.value === 'all') return list
  return list.filter((c) => c.state === filter.value)
})

const FILTER_CHIPS: ReadonlyArray<{ key: Filter; label: string }> = [
  { key: 'all', label: 'All' },
  { key: 'mastered', label: 'Mastered' },
  { key: 'learning', label: 'Learning' },
  { key: 'due', label: 'Due' },
  { key: 'not_started', label: 'New' },
]

function countFor(key: Filter): number {
  if (key === 'all') return progress.summary?.total ?? cases.list.length
  return counts.value?.[key] ?? 0
}

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

function goCase(id: string) {
  router.push(`/cases/${id}`)
}
</script>

<template>
  <main class="page">
    <header class="head">
      <p class="eyebrow">Progress</p>
      <h1 class="title">Where you stand</h1>
    </header>

    <div class="chips">
      <button
        v-for="chip in FILTER_CHIPS"
        :key="chip.key"
        type="button"
        class="chip"
        :class="{ active: filter === chip.key }"
        @click="filter = chip.key"
      >
        <span class="chip-label">{{ chip.label }}</span>
        <span class="chip-count">{{ countFor(chip.key) }}</span>
      </button>
    </div>

    <div v-if="cases.status === 'loading' || progress.status === 'loading'" class="state">
      Loading…
    </div>
    <div
      v-else-if="cases.status === 'error' || progress.status === 'error'"
      class="state error"
    >
      Couldn't load progress.
      <button class="retry" type="button" @click="progress.reload(); cases.refresh()">
        Retry
      </button>
    </div>
    <div v-else-if="filtered.length === 0" class="state">
      No cases match the current filter.
    </div>

    <ul v-else class="list">
      <li
        v-for="c in filtered"
        :key="c.id"
        class="row"
        @click="goCase(c.id)"
      >
        <span class="num">{{ pad2(c.case_number) }}</span>
        <div class="row-meta">
          <p class="row-name">
            <template v-if="c.nickname">{{ c.nickname }}</template>
            <span v-else class="unnamed">Unnamed</span>
          </p>
          <p class="row-sub">
            {{ TIER1_LABELS[c.tier1_tag] ?? c.tier1_tag }}
            <template v-if="c.tags.length > 0"> · {{ c.tags[0] }}</template>
          </p>
        </div>
        <div class="row-state">
          <CaseStatePip :state="c.state" />
          <span class="row-state-label">{{ STATE_LABELS[c.state] }}</span>
        </div>
      </li>
    </ul>

    <section class="stats-card">
      <p class="eyebrow">Stats over time</p>
      <p class="stats-body">Charts and trends — coming soon.</p>
    </section>
  </main>
</template>

<style scoped>
.page {
  background: var(--paper-bg);
  min-height: calc(100vh - 90px);
  padding: 32px 22px 40px;
  color: var(--paper-ink);
}

.head {
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
  font-size: 32px;
  letter-spacing: -0.6px;
  line-height: 1;
  margin: 0;
  color: var(--paper-ink);
}

.chips {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  margin: 0 0 16px;
}

.chip {
  display: flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--paper-rule);
  background: transparent;
  color: var(--paper-ink-muted);
  border-radius: 999px;
  padding: 6px 12px;
  font-family: var(--font-sans);
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
}

.chip.active {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border-color: var(--paper-ink);
}

.chip-count {
  font-size: 11px;
  opacity: 0.75;
}

.state {
  margin-top: 24px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
}

.state.error {
  color: var(--paper-error);
}

.retry {
  margin-left: 8px;
  background: transparent;
  border: 1px solid var(--paper-rule);
  border-radius: 8px;
  padding: 4px 10px;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink);
  cursor: pointer;
}

.list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 10px;
  padding: 10px 12px;
  cursor: pointer;
}

.row:hover {
  border-color: var(--paper-rule);
}

.num {
  font-family: var(--font-serif);
  font-size: 17px;
  letter-spacing: -0.5px;
  color: var(--paper-ink);
  width: 30px;
  text-align: center;
}

.row-meta {
  flex: 1;
  min-width: 0;
}

.row-name {
  font-family: var(--font-serif);
  font-size: 15px;
  color: var(--paper-ink);
  letter-spacing: -0.1px;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unnamed {
  font-style: italic;
  color: var(--paper-ink-faint);
}

.row-sub {
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--paper-ink-faint);
  letter-spacing: 0.2px;
  margin: 2px 0 0;
}

.row-state {
  display: flex;
  align-items: center;
  gap: 6px;
}

.row-state-label {
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--paper-ink-muted);
  letter-spacing: 0.3px;
}

.stats-card {
  margin-top: 24px;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 14px;
  padding: 18px 20px;
}

.stats-body {
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 15px;
  color: var(--paper-ink-muted);
  line-height: 1.4;
  margin: 0;
}
</style>
