// Auth Pinia store.
// Holds current user identity + status. Actions wrap each /auth/* endpoint.
// Stats (streak, last_practice_date) live on a separate progress store later
// — see auth-decisions doc item E.

import { defineStore } from 'pinia'
import { ref } from 'vue'

import { api } from '@/api/client'

export interface User {
  id: string
  email: string
  display_name: string
  pending_email: string | null
  email_verified: boolean
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

  return {
    user,
    status,
    pendingVerificationEmail,
    register,
    verifyEmail,
    resendVerification,
  }
})
