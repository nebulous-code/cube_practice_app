<script setup lang="ts">
defineProps<{ onBack?: () => void }>()
</script>

<template>
  <div class="shell">
    <div v-if="onBack" class="back-row">
      <button class="back" type="button" @click="onBack?.()">← Back</button>
    </div>
    <div class="body" :class="{ 'with-back': !!onBack }">
      <slot />
    </div>
    <div v-if="$slots.footer" class="footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<style scoped>
.shell {
  background: var(--paper-bg);
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  color: var(--paper-ink);
}

.back-row {
  padding: 52px 22px 0;
}

.back {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--paper-ink-muted);
  cursor: pointer;
}

.body {
  flex: 1;
  padding: 64px 26px 0;
}

.body.with-back {
  padding-top: 20px;
}

.footer {
  padding: 20px 26px 28px;
  text-align: center;
}
</style>
