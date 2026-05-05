<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { ApiError } from '@/api/client'
import AuthHeader from '@/components/auth/AuthHeader.vue'
import AuthShell from '@/components/auth/AuthShell.vue'
import PrimaryCTA from '@/components/auth/PrimaryCTA.vue'
import TextLink from '@/components/auth/TextLink.vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const digits = ref<string[]>(['', '', '', '', '', ''])
const inputs = ref<(HTMLInputElement | null)[]>([])
const submitting = ref(false)
const formError = ref<string | null>(null)
const resendNote = ref<string | null>(null)

const code = computed(() => digits.value.join(''))
const valid = computed(() => code.value.length === 6 && /^\d{6}$/.test(code.value))

onMounted(() => {
  // Focus the first box on mount.
  inputs.value[0]?.focus()
})

function setDigit(i: number, raw: string) {
  const onlyDigit = raw.replace(/\D/g, '').slice(-1) ?? ''
  const next = [...digits.value]
  next[i] = onlyDigit
  digits.value = next
  if (onlyDigit && i < 5) {
    nextTick(() => inputs.value[i + 1]?.focus())
  }
}

function onKeydown(i: number, ev: KeyboardEvent) {
  if (ev.key === 'Backspace' && !digits.value[i] && i > 0) {
    nextTick(() => inputs.value[i - 1]?.focus())
  }
  if (ev.key === 'ArrowLeft' && i > 0) {
    ev.preventDefault()
    inputs.value[i - 1]?.focus()
  }
  if (ev.key === 'ArrowRight' && i < 5) {
    ev.preventDefault()
    inputs.value[i + 1]?.focus()
  }
}

function onPaste(ev: ClipboardEvent) {
  const text = ev.clipboardData?.getData('text') ?? ''
  const pasted = text.replace(/\D/g, '').slice(0, 6)
  if (!pasted) return
  ev.preventDefault()
  const next = ['', '', '', '', '', '']
  for (let i = 0; i < pasted.length; i++) next[i] = pasted[i]!
  digits.value = next
  nextTick(() => {
    const focusIdx = Math.min(pasted.length, 5)
    inputs.value[focusIdx]?.focus()
  })
}

async function onSubmit() {
  if (!valid.value || submitting.value) return
  submitting.value = true
  formError.value = null
  resendNote.value = null

  try {
    await auth.verifyEmail(code.value, auth.pendingVerificationEmail)
    // First-time verifications land on the onboarding stub. Existing users
    // (re-verifying after an email change, or signing in on a new device)
    // already have the flag flipped server-side and skip straight to the
    // dashboard. See M5 §5 — trigger lives here, not in a router guard.
    const dest =
      auth.user && auth.user.has_seen_onboarding === false ? '/welcome' : '/practice'
    router.push(dest)
  } catch (err) {
    if (err instanceof ApiError) {
      formError.value = err.message
      // Clear the boxes on a wrong/expired code so the user can retype cleanly.
      if (err.code === 'invalid_code' || err.code === 'code_expired') {
        digits.value = ['', '', '', '', '', '']
        nextTick(() => inputs.value[0]?.focus())
      }
    } else {
      formError.value = 'Something went wrong. Try again.'
    }
  } finally {
    submitting.value = false
  }
}

async function onResend() {
  if (!auth.pendingVerificationEmail) {
    resendNote.value = 'Start over from registration to request a new code.'
    return
  }
  try {
    await auth.resendVerification(auth.pendingVerificationEmail)
    resendNote.value = 'Sent. Check your inbox.'
  } catch (err) {
    resendNote.value = err instanceof ApiError ? err.message : 'Could not resend right now.'
  }
}

function goBack() {
  router.push('/register')
}
</script>

<template>
  <AuthShell :on-back="goBack">
    <AuthHeader
      eyebrow="One more step"
      title="Verify your email"
      :sub="
        auth.pendingVerificationEmail
          ? `We sent a 6-digit code to ${auth.pendingVerificationEmail}.`
          : 'We sent a 6-digit code to your email.'
      "
    />

    <p class="label">Enter code</p>
    <div class="boxes" @paste="onPaste">
      <input
        v-for="(d, i) in digits"
        :key="i"
        :ref="(el) => (inputs[i] = el as HTMLInputElement | null)"
        :value="d"
        inputmode="numeric"
        maxlength="1"
        size="1"
        autocomplete="one-time-code"
        :class="{ filled: !!d }"
        @input="setDigit(i, ($event.target as HTMLInputElement).value)"
        @keydown="onKeydown(i, $event)"
      />
    </div>

    <p v-if="formError" class="form-error">{{ formError }}</p>

    <PrimaryCTA :disabled="!valid || submitting" @click="onSubmit">
      {{ submitting ? 'Verifying…' : 'Verify' }}
    </PrimaryCTA>

    <p class="resend">
      <span>Didn't get it? </span>
      <TextLink @click="onResend">Resend code</TextLink>
    </p>
    <p v-if="resendNote" class="resend-note">{{ resendNote }}</p>
  </AuthShell>
</template>

<style scoped>
.label {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.2px;
  color: var(--paper-ink-faint);
  text-transform: uppercase;
  font-weight: 500;
  margin: 0 0 10px;
}

.boxes {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
}

.boxes input {
  flex: 1;
  min-width: 0;
  width: 0;
  height: 56px;
  text-align: center;
  padding: 0;
  background: var(--paper-card);
  border: 1px solid var(--paper-rule);
  border-radius: 10px;
  font-family: var(--font-mono);
  font-size: 24px;
  color: var(--paper-ink);
  outline: none;
}

.boxes input.filled {
  border-color: var(--paper-ink);
}

.boxes input:focus {
  border-color: var(--paper-ink);
}

.form-error {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-error);
  margin: 8px 0 0;
}

.resend {
  text-align: center;
  margin-top: 22px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
}

.resend-note {
  text-align: center;
  margin-top: 8px;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-faint);
}
</style>
