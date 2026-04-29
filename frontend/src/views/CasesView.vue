<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

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

// Display label for raw Tier 2 tags (mirrors initial_design/src/data.jsx GROUP_LABELS).
const TIER2_LABELS: Record<string, string> = {
  dot: 'Dot',
  T_shapes: 'T-Shapes',
  C_shapes: 'C-Shapes',
  squares: 'Squares',
  lightning_bolts: 'Lightning Bolts',
  I_shapes: 'I-Shapes',
  P_shapes: 'P-Shapes',
  small_L: 'Small L',
  W_shapes: 'W-Shapes',
  fish: 'Fish',
  knight_move: 'Knight Moves',
  awkward_shape: 'Awkward',
  corners_correct: 'Corners Correct',
  solves: 'OCLL / Solves',
}

function tier2Label(raw: string): string {
  return TIER2_LABELS[raw] ?? raw
}

function matchesSearch(c: Case, q: string): boolean {
  if (!q) return true
  const needle = q.toLowerCase()
  if (String(c.case_number).padStart(2, '0').includes(needle)) return true
  if (String(c.case_number).includes(needle)) return true
  if (c.nickname && c.nickname.toLowerCase().includes(needle)) return true
  if (c.algorithm.toLowerCase().includes(needle)) return true
  if (c.tier2_tag && c.tier2_tag.toLowerCase().includes(needle)) return true
  if (tier2Label(c.tier2_tag ?? '').toLowerCase().includes(needle)) return true
  return false
}

function matchesTier1(c: Case): boolean {
  if (tier1Filter.value === 'all') return true
  return c.tier1_tag === tier1Filter.value
}

const filteredGroups = computed(() => {
  return cases.groupedByTier2
    .map(([key, list]) => {
      const filtered = list.filter((c) => matchesTier1(c) && matchesSearch(c, search.value))
      return [key, filtered] as const
    })
    .filter(([, list]) => list.length > 0)
})

const totalCount = computed(() => cases.list.length)
const filteredCount = computed(() =>
  filteredGroups.value.reduce((sum, [, list]) => sum + list.length, 0),
)

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
      <p class="eyebrow">Reference</p>
      <h1 class="title">
        All cases
        <span class="count">{{ totalCount }}</span>
      </h1>
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

    <div v-if="cases.status === 'loading'" class="state">Loading cases…</div>
    <div v-else-if="cases.status === 'error'" class="state error">
      Couldn't load cases. {{ cases.error }}
      <button class="retry" type="button" @click="cases.refresh()">Retry</button>
    </div>
    <div v-else-if="filteredCount === 0" class="state">
      No cases match the current filter.
    </div>

    <section
      v-for="[group, list] in filteredGroups"
      :key="group"
      class="group"
    >
      <header class="group-head">
        <h2 class="group-title">{{ tier2Label(group) }}</h2>
        <p class="group-count">{{ list.length }}</p>
      </header>
      <div class="grid">
        <button
          v-for="c in list"
          :key="c.id"
          type="button"
          class="tile"
          @click="goCase(c.id)"
        >
          <PatternDiagram :pattern="c.pattern" :size="90" />
          <p class="tile-num">{{ pad2(c.case_number) }}</p>
          <p v-if="c.nickname" class="tile-name">{{ c.nickname }}</p>
        </button>
      </div>
    </section>
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

.group {
  margin-top: 24px;
}

.group-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 10px;
}

.group-title {
  font-family: var(--font-serif);
  font-size: 18px;
  font-style: italic;
  letter-spacing: -0.2px;
  color: var(--paper-ink);
  margin: 0;
}

.group-count {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0;
}

.grid {
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
  gap: 4px;
  font-family: var(--font-sans);
  text-align: left;
  color: var(--paper-ink);
  cursor: pointer;
}

.tile:hover {
  border-color: var(--paper-rule);
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
