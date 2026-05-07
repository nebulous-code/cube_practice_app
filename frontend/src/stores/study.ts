// Study Pinia store. Manages the in-flight session queue and the streak.
// In-memory only — reloading the page abandons the session.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { ApiError, api } from '@/api/client'
import {
  dueQueueFromCases,
  reviewCaseInGuest,
} from '@/lib/guest/study-driver'
import { useAuthStore } from '@/stores/auth'
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

// Fisher-Yates in place. Used at session start so the same set of cases
// doesn't drill in the same sequence every day — pattern recognition,
// not positional recognition.
function shuffleInPlace<T>(arr: T[]): T[] {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[arr[i], arr[j]] = [arr[j]!, arr[i]!]
  }
  return arr
}

export const useStudyStore = defineStore('study', () => {
  const queue = ref<Case[]>([])
  const index = ref(0)
  const results = ref<SessionResult[]>([])
  const status = ref<SessionStatus>('idle')
  const streak = ref<Streak>({ count: 0, last_practice_date: null })
  const error = ref<string | null>(null)

  // The unshuffled set the current/last session was started from. Lets
  // "Repeat session" replay the same cards in a freshly-shuffled order.
  const sessionSeed = ref<Case[]>([])

  const currentCase = computed<Case | null>(() => queue.value[index.value] ?? null)
  const remaining = computed(() => Math.max(0, queue.value.length - index.value))

  async function loadDue(): Promise<void> {
    status.value = 'loading'
    error.value = null
    try {
      const auth = useAuthStore()
      if (auth.isGuest && auth.guestState) {
        // Guest mode: derive the due queue from the merged cases list
        // (which the cases store has already computed against the blob).
        const cases = useCasesStore()
        await cases.ensureLoaded()
        queue.value = dueQueueFromCases(cases.list)
        streak.value = {
          count: auth.guestState.streak_count,
          last_practice_date: auth.guestState.last_practice_date,
        }
        status.value = 'idle'
        return
      }
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
    sessionSeed.value = [found]
    index.value = 0
    results.value = []
    status.value = 'in_session'
    return true
  }

  /// Build a multi-card session. With no argument, uses the loaded due
  /// queue (set by `loadDue`). With an explicit `customQueue`, uses that
  /// list as the session source — free-study sessions feed in this way.
  /// The queue is shuffled at session start; the unshuffled source is
  /// retained on `sessionSeed` so `repeatSession()` can re-run the same
  /// set in a fresh order.
  function startSession(customQueue?: Case[]): boolean {
    let source: Case[]
    if (customQueue !== undefined) {
      if (customQueue.length === 0) return false
      source = customQueue.slice()
    } else if (queue.value.length === 0) {
      return false
    } else {
      source = queue.value.slice()
    }
    sessionSeed.value = source.slice()
    queue.value = shuffleInPlace(source.slice())
    index.value = 0
    results.value = []
    status.value = 'in_session'
    return true
  }

  /// Re-run the current session's source set in a fresh shuffled order.
  /// Useful after session-complete for focused free-study drills.
  function repeatSession(): boolean {
    if (sessionSeed.value.length === 0) return false
    return startSession(sessionSeed.value)
  }

  async function submitGrade(grade: Grade): Promise<void> {
    const card = currentCase.value
    if (!card || status.value !== 'in_session') return
    const auth = useAuthStore()
    try {
      if (auth.isGuest && auth.guestState) {
        // Guest mode: SM-2 runs locally; persist via the auth store so
        // the debounced disk write coalesces.
        const result = reviewCaseInGuest(auth.guestState, card.case_number, grade)
        auth.updateGuestState((s) => {
          s.progress = result.blob.progress
          s.streak_count = result.blob.streak_count
          s.last_practice_date = result.blob.last_practice_date
        })
        results.value.push({ caseId: card.id, grade })
        streak.value = {
          count: result.blob.streak_count,
          last_practice_date: result.blob.last_practice_date,
        }

        // Reflect the post-review state on the cached Case row by re-
        // running the merge against the freshly-updated blob.
        const cases = useCasesStore()
        if (auth.guestState && cases.list.length > 0) {
          // mergeGuestSettings expects globals; the cases store keeps
          // them around as `guestGlobals`. Re-derive the full list so
          // every case state stays consistent.
          // The cases store doesn't expose guestGlobals directly; do
          // the simpler thing — update just this card's `state`.
          const idx = cases.list.findIndex((c) => c.id === card.id)
          const cached = idx >= 0 ? cases.list[idx] : undefined
          if (cached) {
            const dueDate = result.progress.due_date
            const today = streak.value.last_practice_date ?? ''
            const newState: Case['state'] =
              dueDate <= today
                ? 'due'
                : result.progress.interval_days < 21
                  ? 'learning'
                  : 'mastered'
            cases.list[idx] = { ...cached, state: newState }
          }
        }
        // Touch progress store too so dashboards refresh.
        void useProgressStore().reload()

        index.value += 1
        if (index.value >= queue.value.length) {
          status.value = 'complete'
        }
        return
      }

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
    sessionSeed.value = []
  }

  function $reset() {
    queue.value = []
    index.value = 0
    results.value = []
    status.value = 'idle'
    streak.value = { count: 0, last_practice_date: null }
    error.value = null
    sessionSeed.value = []
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
    repeatSession,
    submitGrade,
    endSession,
    $reset,
  }
})
