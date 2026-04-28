<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { ApiError } from '@/api/client'
import AuthHeader from '@/components/auth/AuthHeader.vue'
import AuthShell from '@/components/auth/AuthShell.vue'
import Field from '@/components/auth/Field.vue'
import PasswordField from '@/components/auth/PasswordField.vue'
import PrimaryCTA from '@/components/auth/PrimaryCTA.vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const queryEmail = typeof route.query.email === 'string' ? route.query.email : ''
const email = ref(queryEmail)
const code = ref('')
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
    email.value.includes('@') &&
    /^\d{6}$/.test(code.value) &&
    password.value.length >= 8 &&
    !passwordMismatch.value,
)

function onCodeInput(raw: string) {
  code.value = raw.replace(/\D/g, '').slice(0, 6)
}

async function onSubmit() {
  if (!valid.value || submitting.value) return
  submitting.value = true
  formError.value = null
  fieldErrors.value = {}

  try {
    await auth.resetPassword(email.value.trim().toLowerCase(), code.value, password.value)
    router.push('/login')
  } catch (err) {
    if (err instanceof ApiError) {
      if (err.code === 'validation') fieldErrors.value = err.fields
      else formError.value = err.message
    } else {
      formError.value = 'Something went wrong. Try again.'
    }
  } finally {
    submitting.value = false
  }
}

function goBack() {
  router.push('/forgot-password')
}
</script>

<template>
  <AuthShell :on-back="goBack">
    <AuthHeader title="Reset password" sub="Enter the code we sent and choose a new password." />

    <form @submit.prevent="onSubmit">
      <Field
        v-model="email"
        label="Email"
        type="email"
        autocomplete="email"
        :error="fieldErrors.email ?? null"
      />
      <Field
        :model-value="code"
        label="Reset code"
        placeholder="6-digit code"
        autocomplete="one-time-code"
        :error="fieldErrors.code ?? null"
        @update:model-value="onCodeInput"
      />
      <PasswordField
        v-model="password"
        label="New password"
        placeholder="At least 8 characters"
        autocomplete="new-password"
        :error="fieldErrors.new_password ?? null"
      />
      <PasswordField
        v-model="confirm"
        label="Confirm new password"
        autocomplete="new-password"
        :error="passwordMismatch ? `Passwords don't match` : null"
      />

      <p v-if="formError" class="form-error">{{ formError }}</p>

      <PrimaryCTA type="submit" :disabled="!valid || submitting">
        {{ submitting ? 'Updating…' : 'Update password' }}
      </PrimaryCTA>
    </form>
  </AuthShell>
</template>

<style scoped>
.form-error {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-error);
  margin: 8px 0 0;
}
</style>
