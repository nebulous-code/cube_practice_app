<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { type Case, type CaseState, useCasesStore } from '@/stores/cases'
import { useStudyStore } from '@/stores/study'

const router = useRouter()
const route = useRoute()
const cases = useCasesStore()
const study = useStudyStore()

onMounted(() => {
  void cases.ensureLoaded()
})

type Tier1Shape = '*' | 'L' | '-' | '+'
type SelectMode = 'only' | 'any-of'

const TIER1_SHAPES: ReadonlyArray<{ key: Tier1Shape; label: string }> = [
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

const ALL_STATE_KEYS: CaseState[] = ['not_started', 'learning', 'due', 'mastered']

const tier1Mode = ref<SelectMode>('only')
const tier1Filter = ref<Set<Tier1Shape>>(new Set())
const tagMode = ref<SelectMode>('any-of')
const tagFilter = ref<Set<string>>(new Set())
const stateMode = ref<SelectMode>('any-of')
const stateFilter = ref<Set<CaseState>>(new Set<CaseState>(ALL_STATE_KEYS))

// Hydrate filters from query string when arriving via the Free-study link
// from CasesView (or PracticeView's standing chips). Empty / missing
// params leave the defaults alone.
{
  const stateParam = route.query.state
  if (typeof stateParam === 'string' && stateParam.length > 0) {
    const tokens = stateParam.split(',').filter((t): t is CaseState =>
      ALL_STATE_KEYS.includes(t as CaseState),
    )
    if (tokens.length > 0) stateFilter.value = new Set(tokens)
  }
  const tagsParam = route.query.tags
  if (typeof tagsParam === 'string' && tagsParam.length > 0) {
    tagFilter.value = new Set(tagsParam.split(',').filter((t) => t.length > 0))
  }
  const tier1Param = route.query.tier1
  if (typeof tier1Param === 'string' && ['*', 'L', '-', '+'].includes(tier1Param)) {
    tier1Filter.value = new Set([tier1Param as Tier1Shape])
  }
}

// In "only" mode a click is exclusive (replaces the set); clicking the
// already-selected chip clears it. In "any-of" mode a click toggles
// membership. Empty set means "no filter".
function toggleInSet<T>(set: Set<T>, value: T, mode: SelectMode): Set<T> {
  const next = new Set(set)
  if (mode === 'only') {
    if (next.has(value) && next.size === 1) next.clear()
    else {
      next.clear()
      next.add(value)
    }
  } else {
    if (next.has(value)) next.delete(value)
    else next.add(value)
  }
  return next
}

function toggleTier1(s: Tier1Shape) {
  tier1Filter.value = toggleInSet(tier1Filter.value, s, tier1Mode.value)
}

function clearTier1() {
  tier1Filter.value = new Set()
}

function toggleTier1Mode() {
  tier1Mode.value = tier1Mode.value === 'only' ? 'any-of' : 'only'
}

function toggleTag(tag: string) {
  tagFilter.value = toggleInSet(tagFilter.value, tag, tagMode.value)
}

function toggleTagMode() {
  tagMode.value = tagMode.value === 'only' ? 'any-of' : 'only'
}

function toggleState(s: CaseState) {
  // State has inverted semantics from Primary/Tags: the *full* set means
  // "no filter" (all states allowed). In "only" mode, clicking the
  // already-singleton chip restores the full set rather than emptying.
  if (stateMode.value === 'only') {
    if (stateFilter.value.size === 1 && stateFilter.value.has(s)) {
      stateFilter.value = new Set(ALL_STATE_KEYS)
    } else {
      stateFilter.value = new Set([s])
    }
    return
  }
  const next = new Set(stateFilter.value)
  if (next.has(s)) next.delete(s)
  else next.add(s)
  stateFilter.value = next
}

function toggleStateMode() {
  stateMode.value = stateMode.value === 'only' ? 'any-of' : 'only'
}

const filtered = computed<Case[]>(() => {
  return cases.list.filter((c) => {
    if (tier1Filter.value.size > 0 && !tier1Filter.value.has(c.tier1_tag)) return false
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
  else router.push('/practice')
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
      <p class="section-label">
        Primary shape
        <button
          type="button"
          class="hint mode-toggle"
          :aria-label="`Primary shape filter mode: ${tier1Mode === 'only' ? 'only one at a time' : 'any of'} — tap to toggle`"
          @click="toggleTier1Mode"
        >
          {{ tier1Mode === 'only' ? 'only' : 'any of' }}
        </button>
      </p>
      <div class="chips">
        <button
          type="button"
          class="chip"
          :class="{ active: tier1Filter.size === 0 }"
          @click="clearTier1"
        >
          All
        </button>
        <button
          v-for="chip in TIER1_SHAPES"
          :key="chip.key"
          type="button"
          class="chip"
          :class="{ active: tier1Filter.has(chip.key) }"
          @click="toggleTier1(chip.key)"
        >
          {{ chip.label }}
        </button>
      </div>
    </section>

    <section v-if="cases.allTags.length > 0" class="section">
      <p class="section-label">
        Tags
        <button
          type="button"
          class="hint mode-toggle"
          :aria-label="`Tag filter mode: ${tagMode === 'only' ? 'only one at a time' : 'any of'} — tap to toggle`"
          @click="toggleTagMode"
        >
          {{ tagMode === 'only' ? 'only' : 'any of' }}
        </button>
      </p>
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
      <p class="section-label">
        State
        <button
          type="button"
          class="hint mode-toggle"
          :aria-label="`State filter mode: ${stateMode === 'only' ? 'only one at a time' : 'any of'} — tap to toggle`"
          @click="toggleStateMode"
        >
          {{ stateMode === 'only' ? 'only' : 'any of' }}
        </button>
      </p>
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
    <p v-if="matchCount === 0" class="empty-note">
      No cases match — loosen filters.
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

.mode-toggle {
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  text-decoration: underline;
  text-decoration-color: var(--paper-rule-faint);
  text-underline-offset: 3px;
}

.mode-toggle:hover {
  color: var(--paper-ink-muted);
  text-decoration-color: var(--paper-ink-muted);
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

.empty-note {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-faint);
  margin: -8px 0 14px;
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
