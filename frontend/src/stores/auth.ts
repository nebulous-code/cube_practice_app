// Auth Pinia store.
// Holds current user identity + status. Actions wrap each /auth/* endpoint.
// Stats (streak, last_practice_date) live on a separate progress store later
// — see auth-decisions doc item E.

import { defineStore } from 'pinia'
import { ref } from 'vue'

import { ApiError, api } from '@/api/client'
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

export type AuthStatus = 'loading' | 'guest' | 'authed'

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

  /// Bootstrap promise — guarantees we only fetch /auth/me once on initial
  /// page load and that everyone (App.vue, router guard) waits on the same
  /// inflight request.
  let bootstrapPromise: Promise<void> | null = null

  function bootstrap(): Promise<void> {
    if (bootstrapPromise) return bootstrapPromise
    bootstrapPromise = (async () => {
      const startedAt = Date.now()
      try {
        const response = await api.get<User>('/auth/me')
        user.value = response.data
        status.value = 'authed'
      } catch {
        user.value = null
        status.value = 'guest'
      }

      // Splash holds for at least 800ms so it doesn't flicker on a fast response.
      const elapsed = Date.now() - startedAt
      if (elapsed < 800) {
        await new Promise((resolve) => setTimeout(resolve, 800 - elapsed))
      }
    })()
    return bootstrapPromise
  }

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
    status.value = 'guest'
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
    status.value = 'guest'
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

  /// Mark the post-verification onboarding stub as seen for the current
  /// user. Idempotent on the backend; safe to call from both the "Got it"
  /// and "Skip" exits of OnboardingView.
  async function completeOnboarding(): Promise<void> {
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
      status.value = 'guest'
      return null
    }
  }

  return {
    user,
    status,
    pendingVerificationEmail,
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
  }
})
