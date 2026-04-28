<script setup lang="ts">
import { RouterView } from 'vue-router'

import SplashView from '@/components/SplashView.vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
// Kick off bootstrap as early as possible. The router guard awaits the same
// promise, so navigation pauses on the initial route until we know auth state.
auth.bootstrap()
</script>

<template>
  <SplashView v-if="auth.status === 'loading'" />
  <RouterView v-else />
</template>
