// Progress Pinia store. Holds the per-state count breakdown + streak
// fetched from `/progress`. Reloaded after every review submission so
// the dashboard "Standing" card and the Progress view stay in sync.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { api } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import { useCasesStore } from '@/stores/cases'
import type { Streak } from '@/stores/study'

export interface ProgressCounts {
  not_started: number
  learning: number
  due: number
  mastered: number
}

export interface ProgressSummary {
  summary: ProgressCounts
  total: number
  streak: Streak
}

export type ProgressStatus = 'idle' | 'loading' | 'ready' | 'error'

export const useProgressStore = defineStore('progress', () => {
  const summary = ref<ProgressSummary | null>(null)
  const status = ref<ProgressStatus>('idle')
  const error = ref<string | null>(null)

  let inflight: Promise<void> | null = null

  const total = computed(() => summary.value?.total ?? 0)
  const startedCount = computed(() => {
    const s = summary.value?.summary
    if (!s) return 0
    return s.learning + s.due + s.mastered
  })

  async function ensureLoaded(): Promise<void> {
    if (status.value === 'ready') return
    if (inflight) return inflight
    inflight = fetchSummary()
    return inflight
  }

  async function reload(): Promise<void> {
    inflight = fetchSummary()
    return inflight
  }

  async function fetchSummary(): Promise<void> {
    status.value = 'loading'
    error.value = null
    try {
      const auth = useAuthStore()
      if (auth.isGuest && auth.guestState) {
        const cases = useCasesStore()
        await cases.ensureLoaded()
        const counts: ProgressCounts = {
          not_started: 0,
          learning: 0,
          due: 0,
          mastered: 0,
        }
        for (const c of cases.list) counts[c.state] += 1
        summary.value = {
          summary: counts,
          total: cases.list.length,
          streak: {
            count: auth.guestState.streak_count,
            last_practice_date: auth.guestState.last_practice_date,
          },
        }
        status.value = 'ready'
        return
      }
      const response = await api.get<ProgressSummary>('/progress')
      summary.value = response.data
      status.value = 'ready'
    } catch (err) {
      status.value = 'error'
      error.value = err instanceof Error ? err.message : 'Failed to load progress.'
    } finally {
      inflight = null
    }
  }

  function $reset() {
    summary.value = null
    status.value = 'idle'
    error.value = null
    inflight = null
  }

  return {
    summary,
    status,
    error,
    total,
    startedCount,
    ensureLoaded,
    reload,
    $reset,
  }
})
