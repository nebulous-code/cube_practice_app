<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import PatternDiagram from '@/components/PatternDiagram.vue'
import { rotatePattern } from '@/lib/pattern'
import { useCasesStore } from '@/stores/cases'
import { type Grade, useStudyStore } from '@/stores/study'

const router = useRouter()
const cases = useCasesStore()
const study = useStudyStore()

const revealed = ref(false)
const submitting = ref(false)

// Reset reveal state when the current card changes (advance to next).
watch(
  () => study.index,
  () => {
    revealed.value = false
  },
)

interface RatingChip {
  key: string
  grade: Grade
  label: string
  hint: string
  fill: string
  border: string
}

// Tinted-on-paper palette ported from the prototype's RATINGS.
const RATINGS: ReadonlyArray<RatingChip> = [
  { key: 'fail', grade: 0, label: 'Fail', hint: 'Missed it', fill: 'rgba(193, 70, 70, 0.10)', border: 'rgba(193, 70, 70, 0.40)' },
  { key: 'hard', grade: 1, label: 'Hard', hint: 'Got it slowly', fill: 'rgba(195, 130, 50, 0.12)', border: 'rgba(195, 130, 50, 0.45)' },
  { key: 'good', grade: 2, label: 'Good', hint: 'Solid recall', fill: 'rgba(60, 110, 160, 0.12)', border: 'rgba(60, 110, 160, 0.45)' },
  { key: 'easy', grade: 3, label: 'Easy', hint: 'Instant', fill: 'rgba(74, 124, 62, 0.12)', border: 'rgba(74, 124, 62, 0.45)' },
]

const ROTATION_LABELS = ['no rotation', 'rotated 90° CW', 'rotated 180°', 'rotated 90° CCW']

// Always re-resolve the current card through the cases store so edits
// made via the Edit button below show their new algorithm/result on
// return to the study session — study.queue holds the references that
// were captured at session start, and Pinia replaces (rather than
// mutates) the entry in cases.list when settings update.
const current = computed(() => {
  const c = study.currentCase
  if (!c) return null
  return cases.byId(c.id) ?? c
})
const queueLen = computed(() => study.queue.length)
const positionLabel = computed(() => `${study.index + 1} of ${queueLen.value}`)

const resultCase = computed(() => {
  if (!current.value?.result_case_id) return null
  return cases.byId(current.value.result_case_id) ?? null
})

const resultPattern = computed(() => {
  if (!current.value) return ''
  if (!resultCase.value) return current.value.pattern
  return rotatePattern(resultCase.value.pattern, current.value.result_rotation)
})

function rotationLabel(rot: number): string {
  return ROTATION_LABELS[((rot % 4) + 4) % 4] ?? ''
}

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

async function onGrade(grade: Grade) {
  if (submitting.value) return
  submitting.value = true
  await study.submitGrade(grade)
  submitting.value = false
}

function onEnd() {
  study.endSession()
  router.push('/practice')
}

function onDone() {
  study.endSession()
  router.push('/practice')
}

function onRepeat() {
  if (!study.repeatSession()) return
  // Component re-mounts only on route change; force a fresh reveal-state
  // by resetting the local refs and bouncing if we're already on /study.
  revealed.value = false
  submitting.value = false
}

function onEdit() {
  if (!current.value) return
  router.push({
    path: `/cases/${current.value.id}`,
    query: { from: 'study' },
  })
}

function dotBg(i: number): string {
  const past = study.results[i] ?? null
  if (past) return RATINGS[past.grade]?.border ?? 'var(--paper-ink)'
  if (i === study.index && study.status === 'in_session') return 'var(--paper-ink)'
  return 'var(--paper-rule-faint)'
}

function countOf(g: Grade): number {
  return study.results.filter((r) => r.grade === g).length
}

const summaryLine = computed(() => {
  if (study.results.length === 0) return ''
  const avg =
    study.results.reduce((s, r) => s + r.grade, 0) / study.results.length
  if (avg >= 2.3) return 'A confident round.'
  if (avg >= 1.5) return 'Steady progress.'
  if (avg >= 0.8) return 'Grinding away.'
  return 'Tough set — reschedule soon.'
})

// Direct nav into /study with no queue (e.g. user reloaded) — bounce back.
if (study.queue.length === 0 && study.status !== 'complete') {
  router.replace('/practice')
}
</script>

<template>
  <main class="page">
    <header class="top">
      <button v-if="study.status === 'in_session'" type="button" class="end" @click="onEnd">
        × End session
      </button>
      <span v-else class="end-spacer" />
      <span v-if="current" class="position">{{ positionLabel }}</span>
    </header>

    <div v-if="queueLen > 0" class="dots">
      <span
        v-for="(_, i) in study.queue"
        :key="i"
        class="dot"
        :style="{ background: dotBg(i) }"
      />
    </div>

    <section v-if="current" class="body">
      <div class="case-eyebrow">
        <span>Case {{ pad2(current.case_number) }}</span>
        <span v-if="current.nickname" class="dot-sep">·</span>
        <span v-if="current.nickname" class="case-name">{{ current.nickname }}</span>
      </div>

      <div class="diagram">
        <PatternDiagram :pattern="current.pattern" :size="revealed ? 120 : 240" />
      </div>

      <template v-if="!revealed">
        <p class="hint">Execute, then check.</p>
        <p class="hint-sub">
          From a solved yellow top, apply your algorithm and verify the resulting
          shape matches.
        </p>
        <button class="primary" type="button" @click="revealed = true">
          Reveal answer
        </button>
      </template>

      <template v-else>
        <div class="reveal-block">
          <p class="reveal-eyebrow">Should become</p>
          <div class="result-card">
            <PatternDiagram :pattern="resultPattern" :size="78" />
            <div class="result-meta">
              <p class="result-num">
                Case
                {{ pad2(current.result_case_number ?? current.case_number) }}
                <span v-if="resultCase?.nickname" class="result-name">
                  · {{ resultCase.nickname }}
                </span>
              </p>
              <p class="result-rotation">{{ rotationLabel(current.result_rotation) }}</p>
            </div>
          </div>
        </div>

        <div class="reveal-block">
          <p class="reveal-eyebrow">How did it go?</p>
          <div class="grade-grid">
            <button
              v-for="r in RATINGS"
              :key="r.key"
              type="button"
              class="grade"
              :style="{ background: r.fill, borderColor: r.border }"
              :disabled="submitting"
              @click="onGrade(r.grade)"
            >
              <span class="grade-label">{{ r.label }}</span>
              <span class="grade-hint">{{ r.hint }}</span>
            </button>
          </div>
        </div>

        <div class="reveal-block">
          <div class="reveal-head">
            <p class="reveal-eyebrow">Algorithm</p>
            <button type="button" class="edit-link" @click="onEdit">
              Edit →
            </button>
          </div>
          <pre class="algorithm">{{ current.algorithm }}</pre>
        </div>
      </template>
    </section>

    <section v-else-if="study.status === 'complete'" class="complete">
      <p class="reveal-eyebrow">Session complete</p>
      <p class="complete-count">{{ study.results.length }} cases</p>
      <p class="complete-sub">{{ summaryLine }}</p>

      <div class="tally">
        <div v-for="r in RATINGS" :key="r.key" class="tally-row">
          <span class="tally-dot" :style="{ background: r.border }" />
          <span class="tally-label">{{ r.label }}</span>
          <span class="tally-count">{{ countOf(r.grade) }}</span>
        </div>
      </div>

      <div class="complete-actions">
        <button class="primary done" type="button" @click="onRepeat">
          Repeat session
        </button>
        <button class="ghost done" type="button" @click="onDone">
          Back to practice
        </button>
      </div>
    </section>
  </main>
</template>

<style scoped>
.page {
  background: var(--paper-bg);
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  color: var(--paper-ink);
  padding: 0 20px 24px;
}

.top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 56px 0 10px;
}

.end {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  cursor: pointer;
}

.end-spacer {
  width: 1px;
}

.position {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
}

.dots {
  display: flex;
  gap: 3px;
  padding: 0 0 14px;
}

.dot {
  flex: 1;
  height: 3px;
  border-radius: 2px;
}

.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 8px;
}

.case-eyebrow {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-sans);
  font-size: 12px;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--paper-ink-muted);
  margin-bottom: 10px;
}

.dot-sep {
  color: var(--paper-ink-faint);
}

.case-name {
  color: var(--paper-ink-faint);
  text-transform: none;
  letter-spacing: 0;
  font-style: italic;
}

.diagram {
  margin: 12px 0 22px;
}

.hint {
  font-family: var(--font-serif);
  font-size: 22px;
  font-style: italic;
  color: var(--paper-ink-muted);
  margin: 0 0 6px;
  text-align: center;
}

.hint-sub {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-faint);
  text-align: center;
  max-width: 280px;
  margin: 0 auto 24px;
  line-height: 1.5;
}

.primary {
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: none;
  border-radius: 12px;
  padding: 14px 22px;
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  width: 100%;
  max-width: 320px;
}

.primary:hover {
  opacity: 0.92;
}

.reveal-block {
  width: 100%;
  margin-bottom: 16px;
}

.reveal-eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 6px;
}

.reveal-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 6px;
}

.reveal-head .reveal-eyebrow {
  margin: 0;
}

.edit-link {
  background: transparent;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--paper-accent);
  font-weight: 600;
  cursor: pointer;
}

.edit-link:hover {
  opacity: 0.8;
}

.algorithm {
  font-family: var(--font-mono);
  font-size: 17px;
  line-height: 1.6;
  letter-spacing: 0.3px;
  color: var(--paper-ink);
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 10px;
  padding: 10px 14px;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.result-card {
  display: flex;
  gap: 14px;
  align-items: center;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 10px;
  padding: 12px;
}

.result-meta {
  flex: 1;
  min-width: 0;
}

.result-num {
  font-family: var(--font-serif);
  font-size: 18px;
  color: var(--paper-ink);
  line-height: 1.2;
  margin: 0 0 2px;
}

.result-name {
  color: var(--paper-ink-muted);
  font-style: italic;
}

.result-rotation {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-faint);
  margin: 0;
}

.grade-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.grade {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 10px 14px;
  border-radius: 12px;
  border: 1px solid;
  cursor: pointer;
  text-align: left;
}

.grade:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.grade-label {
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.1px;
  color: var(--paper-ink);
}

.grade-hint {
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--paper-ink-muted);
  margin-top: 2px;
}

.complete {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 0;
}

.complete-count {
  font-family: var(--font-serif);
  font-size: 54px;
  letter-spacing: -1px;
  line-height: 1;
  margin: 0 0 6px;
}

.complete-sub {
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 19px;
  color: var(--paper-ink-muted);
  margin: 0 0 32px;
}

.tally {
  width: 100%;
  max-width: 320px;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: 14px;
  padding: 18px;
  margin-bottom: 24px;
}

.tally-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
}

.tally-dot {
  width: 8px;
  height: 8px;
  border-radius: 4px;
}

.tally-label {
  flex: 1;
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--paper-ink);
}

.tally-count {
  font-family: var(--font-serif);
  font-size: 18px;
  color: var(--paper-ink);
}

.done {
  max-width: 320px;
}

.complete-actions {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
  align-items: center;
}

.ghost {
  background: transparent;
  color: var(--paper-ink);
  border: 1px solid var(--paper-rule);
  border-radius: 12px;
  padding: 13px 22px;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  width: 100%;
  max-width: 320px;
}

.ghost:hover {
  border-color: var(--paper-ink);
}
</style>
