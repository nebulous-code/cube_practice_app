<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { ApiError } from '@/api/client'
import AuthHeader from '@/components/auth/AuthHeader.vue'
import AuthShell from '@/components/auth/AuthShell.vue'
import Field from '@/components/auth/Field.vue'
import PrimaryCTA from '@/components/auth/PrimaryCTA.vue'
import TextLink from '@/components/auth/TextLink.vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const email = ref('')
const submitting = ref(false)
const sent = ref(false)
const formError = ref<string | null>(null)

const valid = computed(() => email.value.includes('@'))

async function onSubmit() {
  if (!valid.value || submitting.value) return
  submitting.value = true
  formError.value = null

  try {
    await auth.forgotPassword(email.value.trim().toLowerCase())
    sent.value = true
  } catch (err) {
    formError.value = err instanceof ApiError ? err.message : 'Try again in a moment.'
  } finally {
    submitting.value = false
  }
}

function goEnterCode() {
  router.push({ path: '/reset-password', query: { email: email.value.trim().toLowerCase() } })
}

function goBack() {
  router.push('/login')
}

function startOver() {
  sent.value = false
  formError.value = null
}
</script>

<template>
  <AuthShell :on-back="goBack">
    <template v-if="!sent">
      <AuthHeader
        title="Forgot password"
        sub="Enter your account email and we'll send you a code to reset it."
      />

      <form @submit.prevent="onSubmit">
        <Field
          v-model="email"
          label="Email"
          type="email"
          placeholder="you@example.com"
          autofocus
          autocomplete="email"
        />

        <p v-if="formError" class="form-error">{{ formError }}</p>

        <PrimaryCTA type="submit" :disabled="!valid || submitting">
          {{ submitting ? 'Sending…' : 'Send reset code' }}
        </PrimaryCTA>
      </form>
    </template>

    <template v-else>
      <AuthHeader
        eyebrow="Check your inbox"
        title="Reset link sent"
        :sub="
          `If an account exists for ${email}, you'll receive a 6-digit code shortly.`
        "
      />

      <PrimaryCTA @click="goEnterCode">Enter code</PrimaryCTA>

      <p class="row">
        <TextLink @click="startOver">Use a different email</TextLink>
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

.row {
  text-align: center;
  margin-top: 18px;
}
</style>
