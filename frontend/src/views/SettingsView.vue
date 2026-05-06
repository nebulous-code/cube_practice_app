<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue'
import { RouterLink, useRouter } from 'vue-router'

import { ApiError } from '@/api/client'
import Field from '@/components/auth/Field.vue'
import PasswordField from '@/components/auth/PasswordField.vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

// ─── Profile (display_name + email) ──────────────────────────────────────────
const displayName = ref('')
const email = ref('')
const profileSaving = ref(false)
const profileNote = ref<string | null>(null)
const profileError = ref<string | null>(null)
const profileFieldErrors = ref<Record<string, string>>({})

watchEffect(() => {
  if (auth.user) {
    displayName.value = auth.user.display_name
    email.value = auth.user.email
  }
})

const profileDirty = computed(
  () =>
    !!auth.user &&
    (displayName.value.trim() !== auth.user.display_name ||
      email.value.trim().toLowerCase() !== auth.user.email),
)

async function onSaveProfile() {
  if (!profileDirty.value || profileSaving.value || !auth.user) return
  profileSaving.value = true
  profileNote.value = null
  profileError.value = null
  profileFieldErrors.value = {}

  const payload: { display_name?: string; email?: string } = {}
  if (displayName.value.trim() !== auth.user.display_name) {
    payload.display_name = displayName.value.trim()
  }
  if (email.value.trim().toLowerCase() !== auth.user.email) {
    payload.email = email.value.trim().toLowerCase()
  }

  try {
    const updated = await auth.updateProfile(payload)
    if (updated.pending_email) {
      profileNote.value = `Verification code sent to ${updated.pending_email}.`
    } else {
      profileNote.value = 'Saved.'
    }
  } catch (err) {
    if (err instanceof ApiError) {
      if (err.code === 'validation') profileFieldErrors.value = err.fields
      else profileError.value = err.message
    } else {
      profileError.value = 'Something went wrong. Try again.'
    }
  } finally {
    profileSaving.value = false
  }
}

// ─── Change password ─────────────────────────────────────────────────────────
const currentPw = ref('')
const newPw = ref('')
const confirmPw = ref('')
const pwSaving = ref(false)
const pwNote = ref<string | null>(null)
const pwError = ref<string | null>(null)
const pwFieldErrors = ref<Record<string, string>>({})

const pwMismatch = computed(
  () => confirmPw.value.length > 0 && newPw.value !== confirmPw.value,
)
const pwValid = computed(
  () => currentPw.value.length > 0 && newPw.value.length >= 8 && !pwMismatch.value,
)

async function onChangePassword() {
  if (!pwValid.value || pwSaving.value) return
  pwSaving.value = true
  pwNote.value = null
  pwError.value = null
  pwFieldErrors.value = {}

  try {
    await auth.changePassword(currentPw.value, newPw.value)
    pwNote.value = 'Password updated.'
    currentPw.value = ''
    newPw.value = ''
    confirmPw.value = ''
  } catch (err) {
    if (err instanceof ApiError) {
      if (err.code === 'validation') pwFieldErrors.value = err.fields
      else pwError.value = err.message
    } else {
      pwError.value = 'Something went wrong. Try again.'
    }
  } finally {
    pwSaving.value = false
  }
}

// ─── Sign out / sign out everywhere ──────────────────────────────────────────
const showSignOutAll = ref(false)
const sooaPassword = ref('')
const sooaSaving = ref(false)
const sooaError = ref<string | null>(null)

async function onSignOut() {
  await auth.logout()
  router.push('/login')
}

async function onSignOutAll() {
  if (sooaPassword.value.length === 0 || sooaSaving.value) return
  sooaSaving.value = true
  sooaError.value = null
  try {
    await auth.signOutAll(sooaPassword.value)
    router.push('/login')
  } catch (err) {
    sooaError.value = err instanceof ApiError ? err.message : 'Try again.'
  } finally {
    sooaSaving.value = false
  }
}

function goBack() {
  router.push('/practice')
}
</script>

<template>
  <main class="page">
    <button class="back" type="button" @click="goBack">← Back</button>

    <header>
      <p class="eyebrow">Settings</p>
      <h1>{{ auth.isGuest ? 'Practicing as a guest' : 'Your account' }}</h1>
    </header>

    <!-- Guest-mode upgrade CTA — visible only in guest mode. -->
    <section v-if="auth.isGuest" class="card guest-card">
      <p class="section-eyebrow">Save your progress</p>
      <p class="guest-body">
        Guest data lives on this device only. Create an account and we'll
        fold every review, override, and tag into it without re-grading.
      </p>
      <button
        type="button"
        class="primary"
        @click="router.push('/upgrade')"
      >
        Create account & save progress →
      </button>
    </section>

    <!-- Account section -->
    <section v-if="!auth.isGuest" class="card">
      <p class="section-eyebrow">Account</p>
      <Field
        v-model="displayName"
        label="Display name"
        :error="profileFieldErrors.display_name ?? null"
      />
      <Field
        v-model="email"
        label="Email"
        type="email"
        autocomplete="email"
        :error="profileFieldErrors.email ?? null"
        :hint="
          auth.user?.pending_email
            ? `Pending change to ${auth.user.pending_email} — enter the code on the verify page.`
            : null
        "
      />

      <p v-if="profileError" class="error">{{ profileError }}</p>
      <p v-if="profileNote" class="note">{{ profileNote }}</p>

      <button
        class="primary"
        type="button"
        :disabled="!profileDirty || profileSaving"
        @click="onSaveProfile"
      >
        {{ profileSaving ? 'Saving…' : 'Save changes' }}
      </button>
    </section>

    <!-- Security section -->
    <section v-if="!auth.isGuest" class="card">
      <p class="section-eyebrow">Security</p>

      <PasswordField
        v-model="currentPw"
        label="Current password"
        autocomplete="current-password"
      />
      <PasswordField
        v-model="newPw"
        label="New password"
        :hint="newPw && newPw.length < 8 ? `${8 - newPw.length} more characters` : 'At least 8 characters'"
        autocomplete="new-password"
        :error="pwFieldErrors.new_password ?? null"
      />
      <PasswordField
        v-model="confirmPw"
        label="Confirm new password"
        autocomplete="new-password"
        :error="pwMismatch ? `Passwords don't match` : null"
      />

      <p v-if="pwError" class="error">{{ pwError }}</p>
      <p v-if="pwNote" class="note">{{ pwNote }}</p>

      <button
        class="primary"
        type="button"
        :disabled="!pwValid || pwSaving"
        @click="onChangePassword"
      >
        {{ pwSaving ? 'Updating…' : 'Update password' }}
      </button>
    </section>

    <!-- About / legal -->
    <section class="card">
      <p class="section-eyebrow">About</p>
      <nav class="legal-links">
        <RouterLink to="/about">About Cube Practice</RouterLink>
        <RouterLink to="/terms">Terms of Service</RouterLink>
        <RouterLink to="/privacy">Privacy Policy</RouterLink>
        <RouterLink to="/acknowledgements">Acknowledgements</RouterLink>
      </nav>
    </section>

    <!-- Sign out / Sign out everywhere — authed only; guests have no session. -->
    <section v-if="!auth.isGuest" class="card">
      <p class="section-eyebrow">Sessions</p>
      <button class="ghost" type="button" @click="onSignOut">Sign out</button>

      <button
        v-if="!showSignOutAll"
        class="danger"
        type="button"
        @click="showSignOutAll = true"
      >
        Sign out everywhere
      </button>

      <div v-else class="confirm-row">
        <p class="confirm-text">
          Confirm with your current password — this signs you out on every device.
        </p>
        <PasswordField
          v-model="sooaPassword"
          label="Current password"
          autocomplete="current-password"
        />
        <p v-if="sooaError" class="error">{{ sooaError }}</p>
        <div class="confirm-actions">
          <button class="ghost" type="button" @click="showSignOutAll = false">Cancel</button>
          <button
            class="danger"
            type="button"
            :disabled="sooaPassword.length === 0 || sooaSaving"
            @click="onSignOutAll"
          >
            {{ sooaSaving ? 'Signing out…' : 'Sign out everywhere' }}
          </button>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
.page {
  min-height: 100vh;
  background: var(--paper-bg);
  padding: 52px 22px 40px;
}

.back {
  background: none;
  border: none;
  padding: 0;
  margin-bottom: 16px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  cursor: pointer;
}

header {
  margin-bottom: 24px;
}

.eyebrow,
.section-eyebrow {
  font-family: var(--font-sans);
  font-size: 11px;
  letter-spacing: 1.6px;
  text-transform: uppercase;
  color: var(--paper-ink-faint);
  font-weight: 500;
  margin: 0 0 var(--space-3);
}

h1 {
  font-family: var(--font-serif);
  font-size: 30px;
  letter-spacing: -0.6px;
  color: var(--paper-ink);
  margin: 0;
}

.card {
  background: var(--paper-card);
  border: 1px solid var(--paper-rule-faint);
  border-radius: var(--radius-lg);
  padding: var(--space-5);
  margin-bottom: var(--space-5);
}

.primary {
  width: 100%;
  margin-top: 8px;
  background: var(--paper-ink);
  color: var(--paper-bg);
  border: none;
  border-radius: 12px;
  padding: 13px;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.ghost {
  width: 100%;
  background: transparent;
  color: var(--paper-ink);
  border: 1px solid var(--paper-rule);
  border-radius: 12px;
  padding: 12px;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  margin-bottom: 8px;
}

.danger {
  width: 100%;
  background: transparent;
  color: var(--paper-error);
  border: 1px solid var(--paper-rule);
  border-radius: 12px;
  padding: 12px;
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.danger:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.error {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-error);
  margin: 0 0 8px;
}

.note {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--paper-ink-muted);
  margin: 0 0 8px;
}

.confirm-row {
  margin-top: 12px;
  padding: 12px;
  background: var(--paper-bg-alt);
  border-radius: 10px;
}

.confirm-text {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  line-height: 1.4;
  margin: 0 0 12px;
}

.confirm-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.confirm-actions .ghost,
.confirm-actions .danger {
  margin-bottom: 0;
}

.legal-links {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.legal-links a {
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--paper-ink);
  text-decoration: none;
}

.guest-card {
  border-color: var(--paper-rule);
}

.guest-body {
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.55;
  color: var(--paper-ink-muted);
  margin: 0 0 12px;
}

.legal-links a:hover {
  text-decoration: underline;
  text-underline-offset: 2px;
}
</style>
