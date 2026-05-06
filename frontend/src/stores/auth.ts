// Auth Pinia store.
// Holds current user identity + status. Actions wrap each /auth/* endpoint.
// Stats (streak, last_practice_date) live on a separate progress store later
// — see auth-decisions doc item E.
//
// M6 split the meaning of "guest": the type now distinguishes "anon" (no
// session, no localStorage blob) from "guest" (running off a localStorage
// blob in guest mode). See docs/milestones/06_guest_mode.md §2.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { ApiError, api } from '@/api/client'
import {
  clearGuestState,
  flushGuestState,
  loadGuestState,
  saveGuestState,
} from '@/lib/guest/storage'
import { createInitialState, type GuestState } from '@/lib/guest/state'
import { useCasesStore } from '@/stores/cases'
import { useProgressStore } from '@/stores/progress'
import { useStudyStore } from '@/stores/study'

/// Wipe every store that caches data scoped to the current user. Called
/// on every login/logout/sign-out-all/verify boundary so a second user
/// signing into the same browser never sees the previous user's cache.
function resetUserScopedStores() {
  useCasesStore().$reset()
  useStudyStore().$reset()
  useProgressStore().$reset()
}

export interface User {
  id: string
  email: string
  display_name: string
  pending_email: string | null
  email_verified: boolean
  has_seen_onboarding: boolean
}

/// `'anon'` = no session, no guest blob — a fresh visitor.
/// `'guest'` = running off a localStorage blob (no server identity).
/// `'authed'` = real session.
export type AuthStatus = 'loading' | 'anon' | 'guest' | 'authed'

export interface RegisterPayload {
  display_name: string
  email: string
  password: string
  recaptcha_token: string
}

export interface RegisterResponse {
  id: string
  email: string
  display_name: string
  email_verified: boolean
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const status = ref<AuthStatus>('loading')
  /// Email currently awaiting verification — set after register so the
  /// VerifyEmail screen knows where the code was sent.
  const pendingVerificationEmail = ref<string | null>(null)

  /// Active guest-mode blob. Mirrors what's in localStorage; mutated via
  /// `updateGuestState()` which also schedules a debounced disk write.
  const guestState = ref<GuestState | null>(null)

  /// Set by bootstrap when /auth/me succeeds AND a localStorage blob exists.
  /// The AppShell listens and renders <GuestMergePrompt> exactly once;
  /// merge or discard clears it (see D10).
  const pendingMergePrompt = ref<GuestState | null>(null)

  const isAuthed = computed(() => status.value === 'authed')
  const isGuest = computed(() => status.value === 'guest')
  const isAnon = computed(() => status.value === 'anon')

  /// Bootstrap promise — guarantees we only fetch /auth/me once on initial
  /// page load and that everyone (App.vue, router guard) waits on the same
  /// inflight request.
  let bootstrapPromise: Promise<void> | null = null

  function bootstrap(): Promise<void> {
    if (bootstrapPromise) return bootstrapPromise
    bootstrapPromise = (async () => {
      const startedAt = Date.now()
      const blob = loadGuestState()
      try {
        const response = await api.get<User>('/auth/me')
        user.value = response.data
        status.value = 'authed'
        // Authed-on-a-device-with-guest-data: queue the merge prompt.
        // Don't load the blob into guestState — guest drivers shouldn't
        // run while we're in 'authed' mode.
        if (blob) pendingMergePrompt.value = blob
      } catch {
        user.value = null
        if (blob) {
          guestState.value = blob
          status.value = 'guest'
        } else {
          status.value = 'anon'
        }
      }

      // Splash holds for at least 800ms so it doesn't flicker on a fast response.
      const elapsed = Date.now() - startedAt
      if (elapsed < 800) {
        await new Promise((resolve) => setTimeout(resolve, 800 - elapsed))
      }
    })()
    return bootstrapPromise
  }

  // ─── Guest-mode actions ───────────────────────────────────────────────

  /// Enter guest mode for the first time — invoked by the "Continue as
  /// guest" entries on LandingView and LoginView (D7). Creates a fresh
  /// blob, flips status, persists. Subsequent visits hit the bootstrap
  /// blob path instead.
  function startGuestMode(): GuestState {
    const blob = createInitialState()
    guestState.value = blob
    status.value = 'guest'
    saveGuestState(blob)
    return blob
  }

  /// Mutate the guest blob and schedule a debounced write. Centralized
  /// so callers don't accidentally write to localStorage without
  /// updating the in-memory ref.
  function updateGuestState(mutator: (s: GuestState) => void): void {
    if (!guestState.value) return
    mutator(guestState.value)
    saveGuestState(guestState.value)
  }

  /// Merge the queued guest blob into the now-authed user's account by
  /// hitting `POST /auth/merge-guest-state`. Clears the blob + prompt on
  /// success; the prompt also disappears on a network error so the user
  /// can't get stuck (the original blob stays in localStorage for retry
  /// if we want to surface it again).
  async function mergeGuestState(): Promise<void> {
    if (!pendingMergePrompt.value) return
    const blob = pendingMergePrompt.value
    await api.post('/auth/merge-guest-state', { guest_state: blob })
    pendingMergePrompt.value = null
    clearGuestState()
  }

  /// Discard the queued guest blob without merging. Same exit point for
  /// the "Discard" button in <GuestMergePrompt>.
  function discardGuestState(): void {
    pendingMergePrompt.value = null
    clearGuestState()
  }

  /// Register variant that bundles the current guest blob in the request
  /// body so the backend imports the data into the new user's rows. Used
  /// by `<GuestUpgradeScreen>` (D9). On 200 the local blob is cleared
  /// and the route layer routes to /verify-email.
  async function upgradeFromGuest(payload: RegisterPayload): Promise<RegisterResponse> {
    if (!guestState.value) {
      // No blob → behave like a normal register; saves a special-case
      // branch in the upgrade screen if the user got there via an
      // exotic route.
      return register(payload)
    }
    // Flush any pending debounced write so the blob the backend sees
    // matches the user's last edit.
    flushGuestState()
    const response = await api.post<RegisterResponse>('/auth/register', {
      ...payload,
      guest_state: guestState.value,
    })
    pendingVerificationEmail.value = response.data.email
    clearGuestState()
    guestState.value = null
    // Don't flip status to 'anon' here — the user still needs to verify
    // email. Once they do, verifyEmail() flips to 'authed'.
    return response.data
  }

  // ─── Normal auth actions (unchanged behavior, anon-flavored statuses) ──

  async function register(payload: RegisterPayload): Promise<RegisterResponse> {
    const response = await api.post<RegisterResponse>('/auth/register', payload)
    pendingVerificationEmail.value = response.data.email
    return response.data
  }

  /// Verify a 6-digit code. Email is required for the unauthenticated
  /// (initial registration) path; ignored when an authed session is in play.
  async function verifyEmail(code: string, email?: string | null): Promise<User> {
    const body: { code: string; email?: string } = { code }
    if (email) body.email = email

    const response = await api.post<User>('/auth/verify-email', body)
    const wasAuthed = status.value === 'authed'
    // Initial registration verify is also the moment a new user becomes
    // signed in, so flush any cached data from a previous session here too.
    if (!wasAuthed) resetUserScopedStores()
    user.value = response.data
    status.value = 'authed'
    pendingVerificationEmail.value = null
    return response.data
  }

  /// Resend a verification code. Email is required when no session exists.
  async function resendVerification(email?: string | null): Promise<void> {
    const body: { email?: string } = {}
    if (email) body.email = email
    await api.post('/auth/resend-verification', body)
  }

  async function login(email: string, password: string): Promise<User> {
    try {
      const response = await api.post<User>('/auth/login', { email, password })
      // Wipe any cached data from a previous session before the new user's
      // first request hits the API.
      resetUserScopedStores()
      user.value = response.data
      status.value = 'authed'
      // Login on a device with guest data → surface the merge prompt
      // (same path as bootstrap detecting both at once).
      const blob = loadGuestState()
      if (blob) {
        pendingMergePrompt.value = blob
        guestState.value = null
      }
      return response.data
    } catch (err) {
      // If the account is unverified, surface the email so the verify screen
      // can show "we sent it to <x>" without forcing the user to retype it.
      if (err instanceof ApiError && err.code === 'email_not_verified') {
        pendingVerificationEmail.value = email
      }
      throw err
    }
  }

  async function logout(): Promise<void> {
    try {
      await api.post('/auth/logout')
    } catch {
      // Even if the server-side revoke fails, clear local state — the cookie is
      // either already gone or about to be.
    }
    user.value = null
    status.value = 'anon'
    pendingMergePrompt.value = null
    resetUserScopedStores()
  }

  async function forgotPassword(email: string): Promise<void> {
    await api.post('/auth/forgot-password', { email })
  }

  async function resetPassword(
    email: string,
    code: string,
    newPassword: string,
  ): Promise<void> {
    await api.post('/auth/reset-password', {
      email,
      code,
      new_password: newPassword,
    })
  }

  async function changePassword(
    currentPassword: string,
    newPassword: string,
  ): Promise<void> {
    await api.post('/auth/change-password', {
      current_password: currentPassword,
      new_password: newPassword,
    })
  }

  async function signOutAll(currentPassword: string): Promise<void> {
    await api.post('/auth/sign-out-all', { current_password: currentPassword })
    user.value = null
    status.value = 'anon'
    pendingMergePrompt.value = null
    resetUserScopedStores()
  }

  async function updateProfile(payload: {
    display_name?: string
    email?: string
  }): Promise<User> {
    const response = await api.patch<User>('/auth/me', payload)
    user.value = response.data
    return response.data
  }

  /// Mark the post-verification onboarding stub as seen. In guest mode
  /// the flag lives on the blob (no backend roundtrip — there's no
  /// account yet); in authed mode it hits POST /auth/onboarding-complete.
  /// Idempotent in both modes; safe to call from both the "Got it" and
  /// "Skip" exits of OnboardingView.
  async function completeOnboarding(): Promise<void> {
    if (status.value === 'guest') {
      updateGuestState((s) => {
        s.onboarding_completed = true
      })
      return
    }
    await api.post('/auth/onboarding-complete')
    if (user.value) {
      user.value = { ...user.value, has_seen_onboarding: true }
    }
  }

  /// Refetch /auth/me — useful after backgrounded actions might have changed
  /// pending_email or email_verified state.
  async function refreshMe(): Promise<User | null> {
    try {
      const response = await api.get<User>('/auth/me')
      user.value = response.data
      status.value = 'authed'
      return response.data
    } catch {
      user.value = null
      status.value = guestState.value ? 'guest' : 'anon'
      return null
    }
  }

  return {
    user,
    status,
    pendingVerificationEmail,
    guestState,
    pendingMergePrompt,
    isAuthed,
    isGuest,
    isAnon,
    bootstrap,
    register,
    verifyEmail,
    resendVerification,
    login,
    logout,
    forgotPassword,
    resetPassword,
    changePassword,
    signOutAll,
    updateProfile,
    completeOnboarding,
    refreshMe,
    startGuestMode,
    updateGuestState,
    mergeGuestState,
    discardGuestState,
    upgradeFromGuest,
  }
})
