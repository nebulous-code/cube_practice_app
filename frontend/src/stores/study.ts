// Study Pinia store. Manages the in-flight session queue and the streak.
// In-memory only — reloading the page abandons the session.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { ApiError, api } from '@/api/client'
import { type Case, useCasesStore } from '@/stores/cases'
import { useProgressStore } from '@/stores/progress'

export interface Streak {
  count: number
  last_practice_date: string | null
}

export type SessionStatus =
  | 'idle'
  | 'loading'
  | 'in_session'
  | 'complete'
  | 'error'

export type Grade = 0 | 1 | 2 | 3

interface DueResponse {
  cases: Case[]
  streak: Streak
}

interface ReviewResponse {
  case: Case
  streak: Streak
}

export interface SessionResult {
  caseId: string
  grade: Grade
}

export const useStudyStore = defineStore('study', () => {
  const queue = ref<Case[]>([])
  const index = ref(0)
  const results = ref<SessionResult[]>([])
  const status = ref<SessionStatus>('idle')
  const streak = ref<Streak>({ count: 0, last_practice_date: null })
  const error = ref<string | null>(null)

  const currentCase = computed<Case | null>(() => queue.value[index.value] ?? null)
  const remaining = computed(() => Math.max(0, queue.value.length - index.value))

  async function loadDue(): Promise<void> {
    status.value = 'loading'
    error.value = null
    try {
      const response = await api.get<DueResponse>('/study/due')
      queue.value = response.data.cases
      streak.value = response.data.streak
      status.value = 'idle'
    } catch (err) {
      status.value = 'error'
      error.value = err instanceof Error ? err.message : 'Could not load due cards.'
    }
  }

  /// Build a one-card session for the named case. Used by the "Start
  /// studying" button on the case detail. The case must already be in
  /// the casesStore cache.
  function startSingle(caseId: string): boolean {
    const cases = useCasesStore()
    const found = cases.byId(caseId)
    if (!found) return false
    queue.value = [found]
    index.value = 0
    results.value = []
    status.value = 'in_session'
    return true
  }

  /// Build a multi-card session. With no argument, uses the loaded due
  /// queue (set by `loadDue`). With an explicit `customQueue`, uses that
  /// list verbatim — this is how free-study sessions feed in.
  function startSession(customQueue?: Case[]): boolean {
    if (customQueue !== undefined) {
      if (customQueue.length === 0) return false
      queue.value = customQueue.slice()
    } else if (queue.value.length === 0) {
      return false
    }
    index.value = 0
    results.value = []
    status.value = 'in_session'
    return true
  }

  async function submitGrade(grade: Grade): Promise<void> {
    const card = currentCase.value
    if (!card || status.value !== 'in_session') return
    try {
      const response = await api.post<ReviewResponse>(
        `/study/${card.id}/review`,
        { grade },
      )
      results.value.push({ caseId: card.id, grade })
      streak.value = response.data.streak

      // Reflect the post-review state in the casesStore cache so the
      // browser tile + detail view show the new state without a refetch.
      const cases = useCasesStore()
      const idx = cases.list.findIndex((c) => c.id === response.data.case.id)
      if (idx >= 0) cases.list[idx] = response.data.case

      // Refresh the progress summary so the dashboard / progress views
      // reflect the new state counts.
      void useProgressStore().reload()

      index.value += 1
      if (index.value >= queue.value.length) {
        status.value = 'complete'
      }
    } catch (err) {
      error.value = err instanceof ApiError ? err.message : 'Review failed.'
    }
  }

  function endSession() {
    queue.value = []
    index.value = 0
    results.value = []
    status.value = 'idle'
  }

  function $reset() {
    queue.value = []
    index.value = 0
    results.value = []
    status.value = 'idle'
    streak.value = { count: 0, last_practice_date: null }
    error.value = null
  }

  return {
    queue,
    index,
    results,
    status,
    streak,
    error,
    currentCase,
    remaining,
    loadDue,
    startSingle,
    startSession,
    submitGrade,
    endSession,
    $reset,
  }
})
