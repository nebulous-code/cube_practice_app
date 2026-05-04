<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import CaseStatePip from '@/components/CaseStatePip.vue'
import PatternDiagram from '@/components/PatternDiagram.vue'
import { type Case, useCasesStore } from '@/stores/cases'

const router = useRouter()
const cases = useCasesStore()

onMounted(() => {
  void cases.ensureLoaded()
})

// ─── Filters ────────────────────────────────────────────────────────────────
type Tier1Filter = 'all' | '*' | 'L' | '-' | '+'

const search = ref('')
const tier1Filter = ref<Tier1Filter>('all')

const TIER1_CHIPS: ReadonlyArray<{ key: Tier1Filter; label: string }> = [
  { key: 'all', label: 'All' },
  { key: '*', label: 'Dot' },
  { key: 'L', label: 'L' },
  { key: '-', label: 'Line' },
  { key: '+', label: 'Cross' },
]

// Multi-select tag filter — any-of semantics.
const tagFilter = ref<Set<string>>(new Set())

function toggleTag(tag: string) {
  const next = new Set(tagFilter.value)
  if (next.has(tag)) next.delete(tag)
  else next.add(tag)
  tagFilter.value = next
}

function matchesSearch(c: Case, q: string): boolean {
  if (!q) return true
  const needle = q.toLowerCase()
  if (String(c.case_number).padStart(2, '0').includes(needle)) return true
  if (String(c.case_number).includes(needle)) return true
  if (c.nickname && c.nickname.toLowerCase().includes(needle)) return true
  if (c.algorithm.toLowerCase().includes(needle)) return true
  if (c.tags.some((t) => t.toLowerCase().includes(needle))) return true
  return false
}

function matchesTier1(c: Case): boolean {
  if (tier1Filter.value === 'all') return true
  return c.tier1_tag === tier1Filter.value
}

function matchesTags(c: Case): boolean {
  if (tagFilter.value.size === 0) return true
  return c.tags.some((t) => tagFilter.value.has(t))
}

const filteredCases = computed(() =>
  cases.list
    .filter((c) => matchesTier1(c) && matchesTags(c) && matchesSearch(c, search.value))
    .slice()
    .sort((a, b) => a.case_number - b.case_number),
)

const totalCount = computed(() => cases.list.length)
const filteredCount = computed(() => filteredCases.value.length)

function goCase(id: string) {
  router.push(`/cases/${id}`)
}

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}
</script>

<template>
  <main class="page">
    <header class="head">
      <div class="head-row">
        <div>
          <p class="eyebrow">Reference</p>
          <h1 class="title">
            All cases
            <span class="count">{{ totalCount }}</span>
          </h1>
        </div>
        <button
          type="button"
          class="free-study-btn"
          @click="router.push('/free-study')"
        >
          Free study →
        </button>
      </div>
    </header>

    <div class="search-row">
      <input
        v-model="search"
        type="search"
        class="search"
        placeholder="Search nickname, number, algorithm…"
        autocomplete="off"
      />
    </div>

    <div class="chips">
      <button
        v-for="chip in TIER1_CHIPS"
        :key="chip.key"
        type="button"
        class="chip"
        :class="{ active: tier1Filter === chip.key }"
        @click="tier1Filter = chip.key"
      >
        {{ chip.label }}
      </button>
    </div>

    <div v-if="cases.allTags.length > 0" class="chips tag-chips">
      <button
        v-for="tag in cases.allTags"
        :key="tag"
        type="button"
        class="chip tag-chip"
        :class="{ active: tagFilter.has(tag) }"
        @click="toggleTag(tag)"
      >
        {{ tag }}
      </button>
    </div>

    <div v-if="cases.status === 'loading'" class="state">Loading cases…</div>
    <div v-else-if="cases.status === 'error'" class="state error">
      Couldn't load cases. {{ cases.error }}
      <button class="retry" type="button" @click="cases.refresh()">Retry</button>
    </div>
    <div v-else-if="filteredCount === 0" class="state">
      No cases match the current filter.
    </div>

    <div v-else class="grid">
      <button
        v-for="c in filteredCases"
        :key="c.id"
        type="button"
        class="tile"
        @click="goCase(c.id)"
      >
        <div class="tile-pattern">
          <PatternDiagram :pattern="c.pattern" :size="90" />
          <CaseStatePip :state="c.state" class="tile-pip" />
        </div>
        <p class="tile-num">Case {{ pad2(c.case_number) }}</p>
        <p v-if="c.nickname" class="tile-name">{{ c.nickname }}</p>
      </button>
    </div>
  </main>
</template>

<style scoped>
.page {
  background: var(--paper-bg);
  min-height: 100%;
  padding: 0 22px 90px;
  color: var(--paper-ink);
}

.head {
  padding: 32px 0 8px;
}

.head-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 12px;
}

.free-study-btn {
  background: transparent;
  border: 1px solid var(--paper-rule);
  border-radius: 999px;
  padding: 7px 14px;
  font-family: var(--font-sans);
  font-size: 12px;
  letter-spacing: 0.2px;
  color: var(--paper-ink);
  cursor: pointer;
  white-space: nowrap;
}

.free-study-btn:hover {
  border-color: var(--paper-ink);
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 6px;
}

.title {
  font-family: var(--font-serif);
  font-size: 32px;
  letter-spacing: -0.6px;
  line-height: 1;
  margin: 0;
  color: var(--paper-ink);
}

.count {
  color: var(--paper-ink-faint);
  font-style: italic;
  font-size: 22px;
  margin-left: 6px;
}

.search-row {
  padding: 18px 0 0;
}

.search {
  width: 100%;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 10px;
  padding: 11px 14px;
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--paper-ink);
  outline: none;
  box-sizing: border-box;
}

.search:focus {
  border-color: var(--paper-rule);
}

.chips {
  display: flex;
  gap: 6px;
  padding: 14px 0 0;
  overflow-x: auto;
}

.chip {
  border: 1px solid var(--paper-rule);
  background: transparent;
  color: var(--paper-ink-muted);
  border-radius: 999px;
  padding: 6px 14px;
  font-family: var(--font-sans);
  font-size: 12px;
  letter-spacing: 0.2px;
  cursor: pointer;
  white-space: nowrap;
}

.chip.active {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border-color: var(--paper-ink);
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

.tag-chips {
  padding-top: 8px;
}

.tag-chip {
  font-size: 11px;
  padding: 5px 11px;
}

.grid {
  margin-top: 18px;
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 10px;
}

.tile {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 10px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  font-family: var(--font-sans);
  text-align: center;
  color: var(--paper-ink);
  cursor: pointer;
}

.tile:hover {
  border-color: var(--paper-rule);
}

.tile-pattern {
  position: relative;
}

.tile-pip {
  position: absolute;
  top: -2px;
  right: -2px;
  background: var(--paper-card);
  border-radius: 50%;
  padding: 1px;
}

.tile-num {
  font-family: var(--font-serif);
  font-size: 13px;
  line-height: 1.1;
  margin: 2px 0 0;
  color: var(--paper-ink);
}

.tile-name {
  font-family: var(--font-sans);
  font-size: 10px;
  letter-spacing: 0.2px;
  line-height: 1.1;
  color: var(--paper-ink-muted);
  margin: 0;
  width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
