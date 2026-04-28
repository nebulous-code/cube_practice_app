<script setup lang="ts">
import { useRouter } from 'vue-router'

import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

async function onResend() {
  // Auth'd resend — backend uses session, ignores any body.
  try {
    await auth.resendVerification()
  } catch {
    // Silent — banner stays put either way.
  }
}

function goVerify() {
  router.push('/verify-email')
}
</script>

<template>
  <div v-if="auth.user?.pending_email" class="banner">
    <span class="text">
      Verify your new email <strong>{{ auth.user.pending_email }}</strong> to switch addresses.
    </span>
    <span class="actions">
      <button type="button" class="link" @click="goVerify">Enter code</button>
      <span class="sep">·</span>
      <button type="button" class="link" @click="onResend">Resend</button>
    </span>
  </div>
</template>

<style scoped>
.banner {
  background: var(--paper-accent-bg);
  color: var(--paper-accent);
  padding: 10px 16px;
  font-family: var(--font-sans);
  font-size: 13px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--paper-rule);
}

.text {
  flex: 1 1 auto;
}

.actions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.link {
  background: none;
  border: none;
  padding: 0;
  color: var(--paper-accent);
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.sep {
  opacity: 0.6;
}
</style>
