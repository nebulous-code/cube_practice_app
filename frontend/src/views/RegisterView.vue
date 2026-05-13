<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'

import { ApiError } from '@/api/client'
import AuthHeader from '@/components/auth/AuthHeader.vue'
import AuthShell from '@/components/auth/AuthShell.vue'
import Field from '@/components/auth/Field.vue'
import PasswordField from '@/components/auth/PasswordField.vue'
import PrimaryCTA from '@/components/auth/PrimaryCTA.vue'
import { executeTurnstile } from '@/composables/useTurnstile'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const displayName = ref('')
const email = ref('')
const password = ref('')
const confirm = ref('')
const submitting = ref(false)
const formError = ref<string | null>(null)
const fieldErrors = ref<Record<string, string>>({})

const passwordMismatch = computed(
  () => confirm.value.length > 0 && password.value !== confirm.value,
)
const valid = computed(
  () =>
    displayName.value.trim().length > 0 &&
    email.value.includes('@') &&
    password.value.length >= 8 &&
    !passwordMismatch.value,
)

async function onSubmit() {
  if (!valid.value || submitting.value) return
  submitting.value = true
  formError.value = null
  fieldErrors.value = {}

  try {
    const token = await executeTurnstile('register')
    await auth.register({
      display_name: displayName.value.trim(),
      email: email.value.trim(),
      password: password.value,
      turnstile_token: token,
    })
    router.push('/verify-email')
  } catch (err) {
    if (err instanceof ApiError) {
      if (err.code === 'validation') {
        fieldErrors.value = err.fields
      } else {
        formError.value = err.message
      }
    } else {
      formError.value = 'Something went wrong. Try again.'
    }
  } finally {
    submitting.value = false
  }
}

function goToLogin() {
  router.push('/login')
}
</script>

<template>
  <AuthShell :on-back="goToLogin">
    <AuthHeader title="Create your account" sub="Track progress across devices." />

    <form @submit.prevent="onSubmit">
      <Field
        v-model="displayName"
        label="Display name"
        placeholder="What should we call you?"
        autofocus
        autocomplete="name"
        :error="fieldErrors.display_name ?? null"
      />
      <Field
        v-model="email"
        label="Email"
        type="email"
        placeholder="you@example.com"
        autocomplete="email"
        :error="fieldErrors.email ?? null"
      />
      <PasswordField
        v-model="password"
        placeholder="At least 8 characters"
        autocomplete="new-password"
        :error="fieldErrors.password ?? null"
        :hint="
          password && password.length < 8
            ? `${8 - password.length} more characters`
            : null
        "
      />
      <PasswordField
        v-model="confirm"
        label="Confirm password"
        autocomplete="new-password"
        :error="passwordMismatch ? `Passwords don't match` : null"
      />

      <p v-if="formError" class="form-error">{{ formError }}</p>

      <PrimaryCTA type="submit" :disabled="!valid || submitting">
        {{ submitting ? 'Creating…' : 'Create account' }}
      </PrimaryCTA>
    </form>

    <template #footer>
      <p class="legal">
        By creating an account you agree to our
        <RouterLink to="/terms">Terms</RouterLink> and
        <RouterLink to="/privacy">Privacy Policy</RouterLink>.
      </p>
    </template>
  </AuthShell>
</template>

<style scoped>
.form-error {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-error);
  margin: 8px 0 0;
}

.legal {
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--paper-ink-faint);
  line-height: 1.5;
  margin: 0;
}

.legal :deep(a) {
  color: var(--paper-ink-muted);
  text-decoration: underline;
  text-underline-offset: 2px;
  text-decoration-color: var(--paper-rule-faint);
}

.legal :deep(a:hover) {
  color: var(--paper-ink);
  text-decoration-color: var(--paper-rule);
}
</style>
