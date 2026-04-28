<script setup lang="ts">
import LogoMark from './auth/LogoMark.vue'
</script>

<template>
  <div class="splash">
    <LogoMark :size="88" />
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
