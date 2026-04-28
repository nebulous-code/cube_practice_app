<script setup lang="ts">
import { useRouter } from 'vue-router'

import AuthHeader from '@/components/auth/AuthHeader.vue'
import AuthShell from '@/components/auth/AuthShell.vue'
import TextLink from '@/components/auth/TextLink.vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

function goToRegister() {
  router.push('/register')
}
</script>

<template>
  <AuthShell :on-back="goToRegister">
    <AuthHeader
      eyebrow="Check your inbox"
      title="Verify your email"
      :sub="
        auth.pendingVerificationEmail
          ? `We sent a 6-digit code to ${auth.pendingVerificationEmail}.`
          : 'We sent a 6-digit code to your email.'
      "
    />
    <div class="placeholder">
      <p>
        Code-entry flow lands in the next slice. For now, the verification email is sitting
        in your inbox — confirm it arrived to validate the register loop.
      </p>
      <p>
        Want a different email? <TextLink @click="goToRegister">Start over</TextLink>.
      </p>
    </div>
  </AuthShell>
</template>

<style scoped>
.placeholder {
  font-family: var(--font-serif);
  color: var(--paper-ink-muted);
  line-height: 1.5;
}

.placeholder p {
  margin: 0 0 var(--space-4);
}
</style>
