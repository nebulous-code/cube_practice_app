<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { ApiError } from '@/api/client'
import AuthHeader from '@/components/auth/AuthHeader.vue'
import AuthShell from '@/components/auth/AuthShell.vue'
import Field from '@/components/auth/Field.vue'
import PasswordField from '@/components/auth/PasswordField.vue'
import PrimaryCTA from '@/components/auth/PrimaryCTA.vue'
import TextLink from '@/components/auth/TextLink.vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const email = ref('')
const password = ref('')
const submitting = ref(false)
const formError = ref<string | null>(null)

const valid = computed(() => email.value.includes('@') && password.value.length > 0)

function continueAsGuest() {
  auth.startGuestMode()
  router.push('/welcome')
}

async function onSubmit() {
  if (!valid.value || submitting.value) return
  submitting.value = true
  formError.value = null

  try {
    await auth.login(email.value.trim().toLowerCase(), password.value)
    const nextRaw = route.query.next
    const dest =
      typeof nextRaw === 'string' && nextRaw.startsWith('/') ? nextRaw : '/practice'
    router.push(dest)
  } catch (err) {
    if (err instanceof ApiError) {
      if (err.code === 'email_not_verified') {
        // The store has already stashed the email on pendingVerificationEmail.
        // Trigger a fresh code and route to verify.
        try {
          await auth.resendVerification(email.value.trim().toLowerCase())
        } catch {
          // Swallow — verify screen has its own resend.
        }
        router.push('/verify-email')
        return
      }
      formError.value = err.message
    } else {
      formError.value = 'Something went wrong. Try again.'
    }
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <AuthShell>
    <AuthHeader mark eyebrow="Welcome back" title="Sign in" sub="Pick up where you left off." />

    <form @submit.prevent="onSubmit">
      <Field
        v-model="email"
        label="Email"
        type="email"
        placeholder="you@example.com"
        autofocus
        autocomplete="email"
      />
      <PasswordField
        v-model="password"
        placeholder="••••••••"
        autocomplete="current-password"
      />

      <p class="forgot">
        <TextLink type="button" @click="router.push('/forgot-password')">Forgot password?</TextLink>
      </p>

      <p v-if="formError" class="form-error">{{ formError }}</p>

      <PrimaryCTA type="submit" :disabled="!valid || submitting">
        {{ submitting ? 'Signing in…' : 'Sign in' }}
      </PrimaryCTA>
    </form>

    <template #footer>
      <p class="row">
        New here?
        <TextLink accent @click="router.push('/register')">Create an account</TextLink>
      </p>
      <p class="row guest">
        Just looking around?
        <TextLink @click="continueAsGuest">Continue as guest →</TextLink>
      </p>
    </template>
  </AuthShell>
</template>

<style scoped>
.forgot {
  text-align: right;
  margin: -4px 0 8px;
}

.form-error {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-error);
  margin: 8px 0 0;
}

.row {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  margin: 0;
}

.row.guest {
  margin-top: 8px;
  font-size: 12px;
  color: var(--paper-ink-faint);
}
</style>
