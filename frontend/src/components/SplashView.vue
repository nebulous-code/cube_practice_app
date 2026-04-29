<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'

import LogoMark from './auth/LogoMark.vue'

// Render-free-tier wakeups can take 30+ seconds. The splash shows a
// gentle 90°-with-overshoot rotation cycle so users know the app
// hasn't frozen. Direction is randomized per cycle for visual interest.
const angle = ref(0)
const transition = ref('transform 600ms ease-out')

const OVERSHOOT_MS = 600
const CORRECT_MS = 240
const HOLD_MS = 1000
const OVERSHOOT_DEG = 110
const SETTLE_DEG = 90

let timer: number | undefined

function reducedMotion() {
  return (
    typeof window !== 'undefined' &&
    window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
  )
}

function step() {
  const dir = Math.random() < 0.5 ? 1 : -1
  transition.value = `transform ${OVERSHOOT_MS}ms ease-out`
  angle.value += dir * OVERSHOOT_DEG

  timer = window.setTimeout(() => {
    transition.value = `transform ${CORRECT_MS}ms ease-in-out`
    angle.value -= dir * (OVERSHOOT_DEG - SETTLE_DEG)

    timer = window.setTimeout(step, CORRECT_MS + HOLD_MS)
  }, OVERSHOOT_MS)
}

onMounted(() => {
  if (reducedMotion()) return
  timer = window.setTimeout(step, 400)
})

onBeforeUnmount(() => {
  if (timer !== undefined) window.clearTimeout(timer)
})
</script>

<template>
  <div class="splash">
    <div class="logo" :style="{ transform: `rotate(${angle}deg)`, transition }">
      <LogoMark :size="88" />
    </div>
    <p class="word">Cube Practice</p>
    <p class="tag">a quiet place to drill</p>
  </div>
</template>

<style scoped>
.splash {
  background: var(--paper-bg);
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 22px;
  color: var(--paper-ink);
}

.logo {
  will-change: transform;
}

.word {
  font-family: var(--font-serif);
  font-size: 26px;
  letter-spacing: -0.4px;
  margin: 0;
}

.tag {
  font-family: var(--font-serif);
  font-style: italic;
  font-size: 14px;
  color: var(--paper-ink-faint);
  margin: 0;
  letter-spacing: 0.2px;
}
</style>
