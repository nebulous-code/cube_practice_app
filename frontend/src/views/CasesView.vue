<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import CaseStatePip from '@/components/CaseStatePip.vue'
import EmptyState from '@/components/EmptyState.vue'
import PatternDiagram from '@/components/PatternDiagram.vue'
import { rotatePattern } from '@/lib/pattern'
import { type Case, type CaseState, useCasesStore } from '@/stores/cases'

const router = useRouter()
const route = useRoute()
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

const STATE_CHIPS: ReadonlyArray<{ key: CaseState; label: string }> = [
  { key: 'not_started', label: 'New' },
  { key: 'learning', label: 'Learning' },
  { key: 'due', label: 'Due' },
  { key: 'mastered', label: 'Mastered' },
]

// Multi-select tag and state filters — any-of semantics, matching the
// free-study filter behavior. Empty set = no filter (show everything).
const tagFilter = ref<Set<string>>(new Set())
const stateFilter = ref<Set<CaseState>>(new Set())

function toggleTag(tag: string) {
  const next = new Set(tagFilter.value)
  if (next.has(tag)) next.delete(tag)
  else next.add(tag)
  tagFilter.value = next
}

function toggleState(s: CaseState) {
  const next = new Set(stateFilter.value)
  if (next.has(s)) next.delete(s)
  else next.add(s)
  stateFilter.value = next
}

// Read filter state from the URL on mount — lets the practice screen's
// standing chips deep-link into a filtered cases view (`?state=learning`)
// and lets us round-trip filters through the case-detail page later.
function applyQueryParams() {
  const stateParam = route.query.state
  if (typeof stateParam === 'string' && stateParam.length > 0) {
    const tokens = stateParam.split(',').filter((t): t is CaseState =>
      ['not_started', 'learning', 'due', 'mastered'].includes(t),
    )
    if (tokens.length > 0) stateFilter.value = new Set(tokens)
  }
  const tagsParam = route.query.tags
  if (typeof tagsParam === 'string' && tagsParam.length > 0) {
    tagFilter.value = new Set(tagsParam.split(',').filter((t) => t.length > 0))
  }
  const tier1Param = route.query.tier1
  if (typeof tier1Param === 'string') {
    if (TIER1_CHIPS.some((c) => c.key === tier1Param)) {
      tier1Filter.value = tier1Param as Tier1Filter
    }
  }
}

applyQueryParams()
watch(() => route.fullPath, applyQueryParams)

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

function matchesState(c: Case): boolean {
  if (stateFilter.value.size === 0) return true
  return stateFilter.value.has(c.state)
}

const filteredCases = computed(() =>
  cases.list
    .filter(
      (c) =>
        matchesTier1(c) &&
        matchesTags(c) &&
        matchesState(c) &&
        matchesSearch(c, search.value),
    )
    .slice()
    .sort((a, b) => a.case_number - b.case_number),
)

const totalCount = computed(() => cases.list.length)
const filteredCount = computed(() => filteredCases.value.length)

function goCase(id: string) {
  router.push(`/cases/${id}`)
}

function clearFilters() {
  search.value = ''
  tier1Filter.value = 'all'
  tagFilter.value = new Set()
  stateFilter.value = new Set()
}

function goFreeStudy() {
  // Carry the current filter state through to /free-study so the user
  // can drill into exactly what they're browsing without re-selecting.
  const query: Record<string, string> = {}
  if (tier1Filter.value !== 'all') query.tier1 = tier1Filter.value
  if (tagFilter.value.size > 0) query.tags = [...tagFilter.value].join(',')
  if (stateFilter.value.size > 0) query.state = [...stateFilter.value].join(',')
  router.push({ path: '/free-study', query })
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
          @click="goFreeStudy"
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
        aria-label="Search cases"
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

    <div class="chips state-chips">
      <button
        v-for="chip in STATE_CHIPS"
        :key="chip.key"
        type="button"
        class="chip"
        :class="{ active: stateFilter.has(chip.key) }"
        @click="toggleState(chip.key)"
      >
        {{ chip.label }}
      </button>
    </div>

    <div v-if="cases.status === 'loading'" class="state">Loading cases…</div>
    <div v-else-if="cases.status === 'error'" class="state error">
      Couldn't load cases. {{ cases.error }}
      <button class="retry" type="button" @click="cases.refresh()">Retry</button>
    </div>
    <EmptyState v-else-if="filteredCount === 0" class="empty-card">
      <template #title>No cases match.</template>
      <template #body>
        Loosen the filters or clear them to see the full set.
      </template>
      <template #cta>
        <button type="button" class="cta-clear" @click="clearFilters">
          Clear filters
        </button>
      </template>
    </EmptyState>

    <div v-else class="grid">
      <button
        v-for="c in filteredCases"
        :key="c.id"
        type="button"
        class="tile"
        @click="goCase(c.id)"
      >
        <div class="tile-pattern">
          <PatternDiagram :pattern="rotatePattern(c.pattern, c.display_rotation)" :size="90" />
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

.empty-card {
  margin-top: 24px;
}

.cta-clear {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: 1px solid var(--paper-ink);
  border-radius: var(--radius-md);
  padding: 8px 16px;
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}

.cta-clear:hover {
  background: var(--paper-accent);
  border-color: var(--paper-accent);
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

.state-chips {
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
