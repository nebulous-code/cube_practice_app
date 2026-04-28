<script setup lang="ts">
// Stub login page — the form lands when /auth/login is wired.
import { useRouter } from 'vue-router'

import AuthHeader from '@/components/auth/AuthHeader.vue'
import AuthShell from '@/components/auth/AuthShell.vue'
import PrimaryCTA from '@/components/auth/PrimaryCTA.vue'
import TextLink from '@/components/auth/TextLink.vue'

const router = useRouter()
</script>

<template>
  <AuthShell>
    <AuthHeader mark eyebrow="Welcome back" title="Sign in" sub="Sign-in form is coming next." />
    <PrimaryCTA disabled>Sign in</PrimaryCTA>
    <p class="row">
      New here?
      <TextLink accent @click="router.push('/register')">Create an account</TextLink>
    </p>
  </AuthShell>
</template>

<style scoped>
.row {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  text-align: center;
  margin-top: 32px;
}
</style>
