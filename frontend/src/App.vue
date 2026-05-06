<script setup lang="ts">
import { computed } from 'vue'
import { RouterView } from 'vue-router'

import GuestMergePrompt from '@/components/GuestMergePrompt.vue'
import SplashView from '@/components/SplashView.vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
// Kick off bootstrap as early as possible. The router guard awaits the same
// promise, so navigation pauses on the initial route until we know auth state.
auth.bootstrap()

// Debug hook: append `?splash=hold` to any URL to pin the splash screen
// indefinitely (useful for inspecting the rotation animation without racing
// the bootstrap response). Remove the query param to dismiss.
const splashPinned = computed(() => {
  if (typeof window === 'undefined') return false
  return new URLSearchParams(window.location.search).get('splash') === 'hold'
})
</script>

<template>
  <SplashView v-if="splashPinned || auth.status === 'loading'" />
  <template v-else>
    <RouterView />
    <GuestMergePrompt />
  </template>
</template>
