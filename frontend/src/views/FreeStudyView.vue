<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { type Case, type CaseState, useCasesStore } from '@/stores/cases'
import { useStudyStore } from '@/stores/study'

const router = useRouter()
const cases = useCasesStore()
const study = useStudyStore()

onMounted(() => {
  void cases.ensureLoaded()
})

type Tier1Filter = 'all' | '*' | 'L' | '-' | '+'

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

const tier1 = ref<Tier1Filter>('all')
const tagFilter = ref<Set<string>>(new Set())
const stateFilter = ref<Set<CaseState>>(
  new Set<CaseState>(['not_started', 'learning', 'due', 'mastered']),
)

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

const filtered = computed<Case[]>(() => {
  return cases.list.filter((c) => {
    if (tier1.value !== 'all' && c.tier1_tag !== tier1.value) return false
    if (tagFilter.value.size > 0 && !c.tags.some((t) => tagFilter.value.has(t))) return false
    if (!stateFilter.value.has(c.state)) return false
    return true
  })
})

const matchCount = computed(() => filtered.value.length)

function startSession() {
  if (matchCount.value === 0) return
  if (!study.startSession(filtered.value)) return
  router.push('/study')
}

function cancel() {
  if (window.history.length > 1) router.back()
  else router.push('/')
}
</script>

<template>
  <main class="page">
    <header class="head">
      <button type="button" class="cancel" @click="cancel">× Cancel</button>
      <p class="eyebrow">Free study</p>
      <h1 class="title">Pick what to drill</h1>
    </header>

    <section class="section">
      <p class="section-label">Primary shape</p>
      <div class="chips">
        <button
          v-for="chip in TIER1_CHIPS"
          :key="chip.key"
          type="button"
          class="chip"
          :class="{ active: tier1 === chip.key }"
          @click="tier1 = chip.key"
        >
          {{ chip.label }}
        </button>
      </div>
    </section>

    <section v-if="cases.allTags.length > 0" class="section">
      <p class="section-label">Tags <span class="hint">any of</span></p>
      <div class="chips wrap">
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
    </section>

    <section class="section">
      <p class="section-label">State</p>
      <div class="chips">
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
    </section>

    <p class="count">
      <strong>{{ matchCount }}</strong>
      {{ matchCount === 1 ? 'case matches' : 'cases match' }}
    </p>

    <button
      type="button"
      class="cta"
      :disabled="matchCount === 0"
      @click="startSession"
    >
      Begin session →
    </button>
  </main>
</template>

<style scoped>
.page {
  background: var(--paper-bg);
  min-height: 100vh;
  padding: 32px 22px 40px;
  color: var(--paper-ink);
}

.head {
  position: relative;
  margin-bottom: 22px;
}

.cancel {
  position: absolute;
  top: 0;
  right: 0;
  background: transparent;
  border: none;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  cursor: pointer;
  padding: 4px 8px;
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

.section {
  margin-bottom: 22px;
}

.section-label {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.4px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 8px;
}

.hint {
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 11px;
  letter-spacing: 0.2px;
  text-transform: none;
  color: var(--paper-ink-faint);
  margin-left: 6px;
}

.chips {
  display: flex;
  gap: 6px;
  overflow-x: auto;
}

.chips.wrap {
  flex-wrap: wrap;
  overflow: visible;
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

.tag-chip {
  font-size: 11px;
  padding: 5px 11px;
}

.count {
  font-family: var(--font-serif);
  font-size: 18px;
  font-style: italic;
  color: var(--paper-ink-muted);
  margin: 28px 0 14px;
}

.count strong {
  font-style: normal;
  color: var(--paper-ink);
  font-weight: 500;
  margin-right: 4px;
}

.cta {
  width: 100%;
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: none;
  border-radius: 12px;
  padding: 14px;
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.2px;
  cursor: pointer;
}

.cta:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.cta:not(:disabled):hover {
  opacity: 0.92;
}
</style>
