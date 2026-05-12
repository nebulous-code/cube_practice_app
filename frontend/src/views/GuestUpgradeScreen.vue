<script setup lang="ts">
// Registration variant for guest mode. Sends the same payload as the
// regular register endpoint plus the guest_state blob so the backend
// imports the user's progress in the same transaction. On 200 the local
// blob is cleared and the user is routed to /verify-email — same path
// as a normal registration. See docs/milestones/06_guest_mode.md §6.

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
    await auth.upgradeFromGuest({
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

function goBack() {
  router.push('/practice')
}
</script>

<template>
  <AuthShell :on-back="goBack">
    <AuthHeader
      title="Save your progress"
      sub="Create an account so your work follows you across devices."
    />

    <p class="lede">
      Your existing guest data — schedules, overrides, streak — gets folded
      into the new account in a single step. No re-grading.
    </p>

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
        {{ submitting ? 'Creating…' : 'Create account & save progress' }}
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
.lede {
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.5;
  color: var(--paper-ink-muted);
  margin: 0 0 16px;
}

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
