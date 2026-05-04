<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { ApiError, api } from '@/api/client'
import CaseStatePip from '@/components/CaseStatePip.vue'
import PatternDiagram from '@/components/PatternDiagram.vue'
import { rotatePattern } from '@/lib/pattern'
import { type Case, type SettingsPatch, useCasesStore } from '@/stores/cases'
import { useStudyStore } from '@/stores/study'

const route = useRoute()
const router = useRouter()
const cases = useCasesStore()
const study = useStudyStore()

const ROUTE_ID = computed(() => String(route.params.id ?? ''))

// ─── Loading ────────────────────────────────────────────────────────────────
const current = ref<Case | null>(null)
const loadStatus = ref<'idle' | 'loading' | 'ready' | 'error' | 'not-found'>('idle')
const loadError = ref<string | null>(null)

async function loadCase() {
  loadStatus.value = 'loading'
  loadError.value = null
  // Try the cache first; otherwise fetch the single case.
  const cached = cases.byId(ROUTE_ID.value)
  if (cached) {
    current.value = cached
    loadStatus.value = 'ready'
    return
  }
  try {
    const response = await api.get<Case>(`/cases/${ROUTE_ID.value}`)
    current.value = response.data
    loadStatus.value = 'ready'
  } catch (err) {
    if (err instanceof ApiError && err.status === 404) {
      loadStatus.value = 'not-found'
    } else {
      loadStatus.value = 'error'
      loadError.value = err instanceof Error ? err.message : 'Could not load case.'
    }
  }
}

onMounted(loadCase)
watch(ROUTE_ID, loadCase)

const TIER1_LABELS: Record<string, string> = {
  '*': 'Dot',
  L: 'L',
  '-': 'Line',
  '+': 'Cross',
}

// ─── Edit mode ──────────────────────────────────────────────────────────────
const editing = ref(false)
const draftNickname = ref('')
const draftAlgorithm = ref('')
const draftTags = ref('')
const draftResultCaseNumber = ref<number>(0)
const draftRotation = ref(0)

const saving = ref(false)
const formError = ref<string | null>(null)
const fieldErrors = ref<Record<string, string>>({})

function startEdit() {
  if (!current.value) return
  draftNickname.value = current.value.nickname ?? ''
  draftAlgorithm.value = current.value.algorithm
  draftTags.value = current.value.tags.join(', ')
  draftResultCaseNumber.value = current.value.result_case_number ?? current.value.case_number
  draftRotation.value = current.value.result_rotation
  formError.value = null
  fieldErrors.value = {}
  editing.value = true
}

/// Parse a comma-separated tag input. Strips whitespace; leaves the rest
/// of the normalization (lowercase, dedupe, length cap) to the backend.
function parseTagInput(raw: string): string[] {
  return raw
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
}

/// Compare two tag arrays. Order matters since the merge stores the user's
/// input order verbatim — but post-normalization the server may reorder.
/// For dirty-checking we compare the post-trim user input to the cached
/// list as displayed.
function tagsEqual(a: string[], b: string[]): boolean {
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false
  return true
}

function cancelEdit() {
  editing.value = false
}

const ROTATION_LABELS = ['0°', '90° CW', '180°', '90° CCW']

// Resolve the (frontend-cached) result case for the preview. In edit mode use
// the draft's number; in view mode use the merged result_case_id.
const resultCaseForPreview = computed<Case | null>(() => {
  if (!current.value) return null
  if (editing.value) {
    const target = cases.list.find((c) => c.case_number === draftResultCaseNumber.value)
    return target ?? null
  }
  if (current.value.result_case_id) {
    return cases.byId(current.value.result_case_id) ?? null
  }
  return null
})

const resultPreviewPattern = computed(() => {
  const target = resultCaseForPreview.value
  if (!target) return current.value?.pattern ?? ''
  return rotatePattern(target.pattern, editing.value ? draftRotation.value : current.value?.result_rotation ?? 0)
})

const resultPreviewLabel = computed(() => {
  const n = editing.value ? draftResultCaseNumber.value : current.value?.result_case_number ?? null
  if (n == null) return ''
  return `Case ${String(n).padStart(2, '0')}`
})

const resultRotationLabel = computed(() => {
  const r = editing.value ? draftRotation.value : current.value?.result_rotation ?? 0
  return ROTATION_LABELS[((r % 4) + 4) % 4]
})

// ─── Save ───────────────────────────────────────────────────────────────────
function buildPatch(): SettingsPatch | null {
  if (!current.value) return null
  const c = current.value
  const patch: SettingsPatch = {}
  let touched = false

  const nickTrim = draftNickname.value.trim()
  if (nickTrim !== (c.nickname ?? '')) {
    patch.nickname = nickTrim.length === 0 ? null : nickTrim
    touched = true
  }

  const algTrim = draftAlgorithm.value.trim()
  if (algTrim !== c.algorithm) {
    patch.algorithm = algTrim.length === 0 ? null : algTrim
    touched = true
  }

  const tagsParsed = parseTagInput(draftTags.value)
  if (!tagsEqual(tagsParsed, c.tags)) {
    // Empty input clears the override (server returns to global default).
    patch.tags = tagsParsed.length === 0 ? null : tagsParsed
    touched = true
  }

  // Resolve the chosen result case number to a UUID via the cached list.
  const targetCase = cases.list.find((x) => x.case_number === draftResultCaseNumber.value)
  if (!targetCase) {
    fieldErrors.value = { result_case_id: 'No case with that number.' }
    return null
  }
  if (targetCase.id !== c.result_case_id) {
    patch.result_case_id = targetCase.id
    touched = true
  }
  if (draftRotation.value !== c.result_rotation) {
    patch.result_rotation = draftRotation.value
    touched = true
  }

  return touched ? patch : {}
}

async function onSave() {
  if (!current.value || saving.value) return
  saving.value = true
  formError.value = null
  fieldErrors.value = {}

  const patch = buildPatch()
  if (patch == null) {
    saving.value = false
    return
  }
  if (Object.keys(patch).length === 0) {
    editing.value = false
    saving.value = false
    return
  }

  try {
    const merged = await cases.updateSettings(current.value.id, patch)
    current.value = merged
    editing.value = false
  } catch (err) {
    if (err instanceof ApiError) {
      if (err.code === 'validation') fieldErrors.value = err.fields
      else formError.value = err.message
    } else {
      formError.value = 'Save failed. Try again.'
    }
  } finally {
    saving.value = false
  }
}

// Make sure the cached list is loaded so the result preview can resolve.
onMounted(() => {
  void cases.ensureLoaded()
})

function goBack() {
  if (window.history.length > 1) router.back()
  else router.push('/cases')
}

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

async function onStartStudying() {
  if (!current.value) return
  // Make sure the case is in casesStore so studyStore can find it.
  await cases.ensureLoaded()
  if (!study.startSingle(current.value.id)) return
  router.push('/study')
}
</script>

<template>
  <main class="page">
    <header class="head">
      <button type="button" class="back" @click="goBack">← Back</button>
      <div v-if="!editing" class="actions">
        <button v-if="loadStatus === 'ready'" type="button" class="action" @click="startEdit">
          Edit
        </button>
      </div>
      <div v-else class="actions">
        <button type="button" class="action quiet" @click="cancelEdit">Cancel</button>
        <button type="button" class="action accent" :disabled="saving" @click="onSave">
          {{ saving ? 'Saving…' : 'Save' }}
        </button>
      </div>
    </header>

    <div v-if="loadStatus === 'loading'" class="state">Loading case…</div>
    <div v-else-if="loadStatus === 'not-found'" class="state error">
      Case not found.
      <button class="retry" type="button" @click="router.push('/cases')">Back to all cases</button>
    </div>
    <div v-else-if="loadStatus === 'error'" class="state error">
      {{ loadError }}
      <button class="retry" type="button" @click="loadCase">Retry</button>
    </div>

    <template v-else-if="current">
      <section class="title-row">
        <p class="eyebrow">
          Case {{ pad2(current.case_number) }}
          <template v-if="current.tags.length > 0"> · {{ current.tags[0] }}</template>
        </p>
        <h1 v-if="!editing" class="title">
          <template v-if="current.nickname">{{ current.nickname }}</template>
          <span v-else class="unnamed">Unnamed</span>
        </h1>
        <input
          v-else
          v-model="draftNickname"
          type="text"
          class="title-input"
          placeholder="Give it a nickname…"
          maxlength="80"
        />
        <div v-if="!editing" class="state-row">
          <CaseStatePip :state="current.state" :show-label="true" />
        </div>
      </section>

      <button
        v-if="!editing"
        type="button"
        class="cta primary"
        @click="onStartStudying"
      >
        {{ current.state === 'not_started' ? 'Start studying' : 'Practice now' }}
      </button>

      <!-- Pattern + tags -->
      <section class="card meta-card">
        <PatternDiagram :pattern="current.pattern" :size="120" />
        <div class="meta">
          <p class="section-eyebrow">Primary shape</p>
          <p class="meta-value">{{ TIER1_LABELS[current.tier1_tag] ?? current.tier1_tag }}</p>

          <p class="section-eyebrow">Tags</p>
          <div v-if="!editing" class="tag-row">
            <span v-if="current.tags.length === 0" class="tag-empty">—</span>
            <span v-for="t in current.tags" :key="t" class="tag-chip-display">{{ t }}</span>
          </div>
          <input
            v-else
            v-model="draftTags"
            type="text"
            class="inline-input"
            placeholder="e.g. fish, needs work"
          />
          <p v-if="editing && fieldErrors.tags" class="error">{{ fieldErrors.tags }}</p>
        </div>
      </section>

      <!-- Algorithm -->
      <section class="card">
        <p class="section-eyebrow">Algorithm</p>
        <pre v-if="!editing" class="algorithm">{{ current.algorithm }}</pre>
        <textarea
          v-else
          v-model="draftAlgorithm"
          rows="3"
          class="algorithm-input"
          maxlength="1000"
        />
        <p v-if="fieldErrors.algorithm" class="error">{{ fieldErrors.algorithm }}</p>
      </section>

      <!-- Result preview -->
      <section class="card">
        <p class="section-eyebrow">Result after algorithm</p>
        <div class="result">
          <PatternDiagram :pattern="resultPreviewPattern" :size="92" />
          <div class="result-meta">
            <p class="meta-value">{{ resultPreviewLabel }}</p>
            <p class="rotation-label">{{ resultRotationLabel }}</p>
          </div>
        </div>

        <div v-if="editing" class="result-edit">
          <div class="row">
            <label class="label">
              Result case number
              <input
                v-model.number="draftResultCaseNumber"
                type="number"
                min="1"
                max="57"
                class="num-input"
              />
            </label>
          </div>
          <p v-if="fieldErrors.result_case_id" class="error">{{ fieldErrors.result_case_id }}</p>
          <p class="section-eyebrow rotation-eyebrow">Rotation</p>
          <div class="rot-grid">
            <button
              v-for="r in [0, 1, 2, 3]"
              :key="r"
              type="button"
              class="rot-btn"
              :class="{ active: draftRotation === r }"
              @click="draftRotation = r"
            >
              {{ ROTATION_LABELS[r] }}
            </button>
          </div>
        </div>
      </section>

      <p v-if="current.has_overrides" class="footer-note">
        This case has personal overrides — clear all fields and save to revert to defaults.
      </p>
      <p v-if="formError" class="error">{{ formError }}</p>
    </template>
  </main>
</template>

<style scoped>
.page {
  background: var(--paper-bg);
  min-height: 100vh;
  padding: 0 22px 60px;
  color: var(--paper-ink);
}

.head {
  padding: 52px 0 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.back {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  cursor: pointer;
}

.actions {
  display: flex;
  gap: 12px;
}

.action {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 12px;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--paper-accent);
  cursor: pointer;
  font-weight: 600;
}

.action.quiet {
  color: var(--paper-ink-muted);
  font-weight: 500;
}

.action.accent {
  color: var(--paper-accent);
  font-weight: 600;
}

.action:disabled {
  opacity: 0.4;
  cursor: not-allowed;
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

.title-row {
  margin: 18px 0 14px;
}

.state-row {
  margin-top: 8px;
}

.cta {
  width: 100%;
  margin: 0 0 14px;
  padding: 13px;
  border-radius: 12px;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid var(--paper-ink);
  background: var(--paper-ink);
  color: var(--paper-bg);
}

.cta:hover {
  opacity: 0.92;
}

.eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 4px;
}

.title {
  font-family: var(--font-serif);
  font-size: 28px;
  letter-spacing: -0.5px;
  line-height: 1.05;
  color: var(--paper-ink);
  margin: 0;
}

.unnamed {
  color: var(--paper-ink-faint);
  font-style: italic;
}

.title-input {
  width: 100%;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--paper-rule);
  font-family: var(--font-serif);
  font-size: 26px;
  letter-spacing: -0.4px;
  color: var(--paper-ink);
  padding: 2px 0;
  outline: none;
}

.card {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-lg, 14px);
  padding: 18px;
  margin-bottom: 14px;
}

.meta-card {
  display: flex;
  gap: 18px;
  align-items: flex-start;
}

.meta {
  flex: 1;
  min-width: 0;
}

.section-eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 6px;
}

.meta-value {
  font-family: var(--font-serif);
  font-size: 16px;
  color: var(--paper-ink);
  margin: 0 0 14px;
  font-style: italic;
}

.inline-input {
  width: 100%;
  background: var(--paper-bg);
  border: 1px solid var(--paper-rule);
  border-radius: 6px;
  padding: 6px 8px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink);
  margin: 0 0 14px;
  outline: none;
  box-sizing: border-box;
}

.tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 0 0 14px;
}

.tag-chip-display {
  font-family: var(--font-sans);
  font-size: 12px;
  letter-spacing: 0.2px;
  background: var(--paper-bg);
  border: 1px solid var(--paper-rule-faint);
  color: var(--paper-ink);
  border-radius: 999px;
  padding: 4px 10px;
}

.tag-empty {
  font-family: var(--font-serif);
  font-style: italic;
  color: var(--paper-ink-faint);
}

.algorithm {
  font-family: var(--font-mono);
  font-size: 16px;
  color: var(--paper-ink);
  line-height: 1.6;
  letter-spacing: 0.3px;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.algorithm-input {
  width: 100%;
  background: var(--paper-bg);
  border: 1px solid var(--paper-rule);
  border-radius: 8px;
  padding: 10px 12px;
  font-family: var(--font-mono);
  font-size: 16px;
  color: var(--paper-ink);
  line-height: 1.5;
  letter-spacing: 0.3px;
  outline: none;
  resize: vertical;
  box-sizing: border-box;
}

.result {
  display: flex;
  align-items: center;
  gap: 14px;
}

.result-meta {
  flex: 1;
}

.rotation-label {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-faint);
  margin: 4px 0 0;
}

.result-edit {
  margin-top: 16px;
  border-top: 1px solid var(--paper-rule-faint);
  padding-top: 14px;
}

.row {
  margin-bottom: 12px;
}

.label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
}

.num-input {
  width: 80px;
  padding: 8px 10px;
  border: 1px solid var(--paper-rule);
  border-radius: 8px;
  font-family: var(--font-mono);
  font-size: 15px;
  background: var(--paper-bg);
  color: var(--paper-ink);
  outline: none;
}

.rotation-eyebrow {
  margin-top: 8px;
}

.rot-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr 1fr;
  gap: 6px;
}

.rot-btn {
  padding: 9px 4px;
  border: 1px solid var(--paper-rule);
  background: transparent;
  color: var(--paper-ink);
  border-radius: 8px;
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 0.3px;
  cursor: pointer;
}

.rot-btn.active {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border-color: var(--paper-ink);
}

.footer-note {
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--paper-ink-faint);
  margin: 6px 0 0;
  font-style: italic;
}

.error {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-error);
  margin: 8px 0 0;
}
</style>
