// Cases Pinia store. One source of truth for the merged-and-flattened
// case list. Fetched once on app shell mount via `ensureLoaded()`; PATCH
// responses replace the cached row in place so browser/detail views stay
// consistent without refetching.
//
// API shape mirrors `Case` in backend/src/cases/mod.rs.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { api } from '@/api/client'

export type CaseState = 'not_started' | 'learning' | 'due' | 'mastered'

export interface Case {
  id: string
  solve_stage: string
  puzzle_type: string
  case_number: number
  nickname: string | null
  algorithm: string
  result_case_id: string | null
  result_case_number: number | null
  result_rotation: number
  pattern: string
  tier1_tag: '+' | '-' | 'L' | '*'
  tier2_tag: string | null
  has_overrides: boolean
  state: CaseState
}

export type CasesStatus = 'idle' | 'loading' | 'ready' | 'error'

export interface SettingsPatch {
  nickname?: string | null
  algorithm?: string | null
  result_case_id?: string | null
  result_rotation?: number | null
  tier2_tag?: string | null
}

interface ListResponse {
  cases: Case[]
}

export const useCasesStore = defineStore('cases', () => {
  const list = ref<Case[]>([])
  const status = ref<CasesStatus>('idle')
  const error = ref<string | null>(null)

  // Single-flight: callers can await ensureLoaded() concurrently and share
  // the same in-flight request.
  let inflight: Promise<void> | null = null

  function byId(id: string): Case | undefined {
    return list.value.find((c) => c.id === id)
  }

  /// Cases grouped by Tier 2 tag, with group keys sorted alphabetically and
  /// cases within each group sorted by case_number ASC. Cases with no Tier 2
  /// tag fall under a special `(untagged)` bucket sorted last.
  const groupedByTier2 = computed<Array<[string, Case[]]>>(() => {
    const groups: Record<string, Case[]> = {}
    for (const c of list.value) {
      const key = c.tier2_tag ?? '(untagged)'
      ;(groups[key] ??= []).push(c)
    }
    for (const cases of Object.values(groups)) {
      cases.sort((a, b) => a.case_number - b.case_number)
    }
    return Object.entries(groups).sort(([a], [b]) => {
      if (a === '(untagged)') return 1
      if (b === '(untagged)') return -1
      return a.localeCompare(b, undefined, { sensitivity: 'base' })
    })
  })

  async function ensureLoaded(): Promise<void> {
    if (status.value === 'ready') return
    if (inflight) return inflight
    inflight = (async () => {
      status.value = 'loading'
      error.value = null
      try {
        const response = await api.get<ListResponse>('/cases')
        list.value = response.data.cases
        status.value = 'ready'
      } catch (err) {
        status.value = 'error'
        error.value = err instanceof Error ? err.message : 'Failed to load cases.'
      } finally {
        inflight = null
      }
    })()
    return inflight
  }

  async function refresh(): Promise<void> {
    inflight = null
    status.value = 'idle'
    return ensureLoaded()
  }

  /// PATCH `/cases/:id/settings`. Each patch field uses the same
  /// `undefined | null | value` semantics as the backend: `undefined`
  /// (omit from payload) leaves the override alone; `null` clears it;
  /// a value sets it. The merged response replaces the cached row.
  async function updateSettings(id: string, patch: SettingsPatch): Promise<Case> {
    const response = await api.patch<Case>(`/cases/${id}/settings`, patch)
    const merged = response.data
    const idx = list.value.findIndex((c) => c.id === id)
    if (idx >= 0) list.value[idx] = merged
    return merged
  }

  function $reset() {
    list.value = []
    status.value = 'idle'
    error.value = null
    inflight = null
  }

  return {
    list,
    status,
    error,
    byId,
    groupedByTier2,
    ensureLoaded,
    refresh,
    updateSettings,
    $reset,
  }
})
