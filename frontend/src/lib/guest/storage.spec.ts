// Tests for the guest-blob storage layer. Driven against jsdom's
// localStorage, which Vitest provides via the `jsdom` environment in
// the project's vitest config.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import {
  clearGuestState,
  flushGuestState,
  loadGuestState,
  saveGuestState,
  STORAGE_KEY,
} from './storage'
import { createInitialState } from './state'

describe('guest storage', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('returns null when nothing is stored', () => {
    expect(loadGuestState()).toBeNull()
  })

  it('flush writes the pending state to localStorage', () => {
    const state = createInitialState(new Date('2026-05-12T19:00:00Z'))
    saveGuestState(state)
    flushGuestState()
    const raw = localStorage.getItem(STORAGE_KEY)
    expect(raw).not.toBeNull()
    expect(JSON.parse(raw!)).toEqual(state)
  })

  it('save coalesces rapid mutations within the debounce window', () => {
    const a = createInitialState(new Date('2026-05-12T19:00:00Z'))
    const b = { ...a, streak_count: 5 }
    const c = { ...a, streak_count: 9 }
    saveGuestState(a)
    saveGuestState(b)
    saveGuestState(c)
    // No write yet — the timer hasn't fired.
    expect(localStorage.getItem(STORAGE_KEY)).toBeNull()
    vi.advanceTimersByTime(300)
    // Only the latest landed.
    const final = JSON.parse(localStorage.getItem(STORAGE_KEY)!)
    expect(final.streak_count).toBe(9)
  })

  it('load returns the stored state after a save+flush', () => {
    const state = createInitialState(new Date('2026-05-12T19:00:00Z'))
    state.streak_count = 7
    saveGuestState(state)
    flushGuestState()
    expect(loadGuestState()).toEqual(state)
  })

  it('clear removes the stored blob and cancels any pending write', () => {
    const state = createInitialState(new Date('2026-05-12T19:00:00Z'))
    saveGuestState(state)
    clearGuestState()
    vi.advanceTimersByTime(500)
    expect(localStorage.getItem(STORAGE_KEY)).toBeNull()
  })

  it('load returns null when stored JSON is malformed', () => {
    // Suppress the expected console.warn so test output stays clean.
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    localStorage.setItem(STORAGE_KEY, '{not valid json')
    expect(loadGuestState()).toBeNull()
    warn.mockRestore()
  })

  it('load returns null when the blob version is missing', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ no_version: true }))
    expect(loadGuestState()).toBeNull()
    warn.mockRestore()
  })

  it('load returns null when the blob version is from the future', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ version: 99, settings: {}, progress: {} }),
    )
    expect(loadGuestState()).toBeNull()
    warn.mockRestore()
  })
})
